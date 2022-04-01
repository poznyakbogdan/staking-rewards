use crate::pda_helper::PdaHelper;
use solana_program::clock::UnixTimestamp;
use crate::state::UserStakingState;
use solana_program::program::invoke_signed;
use solana_program::program::invoke;
use crate::state::StakingState;
use solana_program::sysvar::Sysvar;
use solana_program::sysvar::clock::Clock;
use borsh::BorshDeserialize;
use solana_program::{
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    account_info::AccountInfo,
    program_error::ProgramError,
    account_info::next_account_info,
    program_pack::{Pack},
    msg,
    rent::Rent,
};
use crate::instruction::StakingInstruction;
use spl_token::state::Account;

pub struct Processor;

impl Processor {
    const REWARD_RATE: u64 = 100;

    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
        let decoded_data = StakingInstruction::try_from_slice(instruction_data)?;
        match decoded_data {
            StakingInstruction::Init => {
                Self::initialize(program_id, accounts)
            },
            StakingInstruction::Stake { amount } => {
                Self::stake(program_id, accounts, amount)
            },
            StakingInstruction::Unstake { amount } => {
                Self::unstake(program_id, accounts, amount)
            },
            StakingInstruction::GetRewards => {
                Self::get_rewards(program_id, accounts)
            }
        }
    }
    
    fn initialize(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let admin_ai = next_account_info(accounts_iter)?;
        let metadata_pda_ai = next_account_info(accounts_iter)?;
        let staking_token_mint_ai = next_account_info(accounts_iter)?;
        let rewards_token_mint_ai = next_account_info(accounts_iter)?;
        let staking_token_ai = next_account_info(accounts_iter)?;
        let rewards_token_ai = next_account_info(accounts_iter)?;
        let system_program = next_account_info(accounts_iter)?;
        let token_program = next_account_info(accounts_iter)?;

        let clock = Clock::get()?;

        let (metadata_pda, bump_seed) = PdaHelper::find_metadata_pda(staking_token_mint_ai, rewards_token_mint_ai, program_id);

        if *metadata_pda_ai.key != metadata_pda {
            msg!("Metadata passed: {}", metadata_pda_ai.key);
            msg!("Metadata computed: {}", metadata_pda);
            return Err(ProgramError::InvalidAccountData);
        }

        let in_use = !metadata_pda_ai.try_data_is_empty()?;

        if in_use {
            let mut staking_state = StakingState::try_from_slice(&metadata_pda_ai.try_borrow_mut_data()?)?;
            
            staking_state.admin = *admin_ai.key;
            staking_state.staking_token_mint = *staking_token_mint_ai.key;
            staking_state.reward_token_mint = *rewards_token_mint_ai.key;
            staking_state.total_supply = 0;
            staking_state.last_update_timestamp = clock.unix_timestamp;
            staking_state.reward_per_token_stored = 0;
    
            staking_state.pack(&mut metadata_pda_ai.try_borrow_mut_data()?);

            msg!("Reset metadata values");
        } else {
            msg!("Trying to create account");
    
            let create_account_ix = solana_program::system_instruction::create_account(
                &admin_ai.key, 
                &metadata_pda, 
                Rent::get()?.minimum_balance(StakingState::LEN),
                StakingState::LEN as u64, 
                program_id);
    
            invoke_signed(&create_account_ix, &[
                admin_ai.clone(),
                metadata_pda_ai.clone(),
                system_program.clone()
            ], &[
                &[
                    staking_token_mint_ai.key.as_ref(),
                    rewards_token_mint_ai.key.as_ref(),
                    b"metadata", 
                    &[bump_seed]
                ]
            ])?;
            
            msg!("Metadata pda created: {}", metadata_pda_ai.key);
    
            let mut staking_state = StakingState::try_from_slice(&metadata_pda_ai.try_borrow_mut_data()?)?;
            
            staking_state.admin = *admin_ai.key;
            staking_state.staking_token_mint = *staking_token_mint_ai.key;
            staking_state.reward_token_mint = *rewards_token_mint_ai.key;
            staking_state.total_supply = 0;
            staking_state.last_update_timestamp = clock.unix_timestamp;
            staking_state.reward_per_token_stored = 0;
    
            staking_state.pack(&mut metadata_pda_ai.try_borrow_mut_data()?);
            
            msg!("Initialized staking with next values: ");
            msg!("admin: {}", admin_ai.key);
            msg!("staking token mint pubkey: {}", staking_token_mint_ai.key);
            msg!("reward token mint pubkey: {}", rewards_token_mint_ai.key);
            msg!("total staked: {}", 0);
            msg!("last reward timestamp: {}", clock.unix_timestamp);
        };

        let (staking_token_pda, _nonce) = PdaHelper::find_staking_token_pda(staking_token_mint_ai, program_id);
        let (rewards_token_pda, _nonce) = PdaHelper::find_rewards_token_pda(rewards_token_mint_ai, program_id);

        let staking_token_acc = Account::unpack(&staking_token_ai.try_borrow_data()?)?;
        let rewards_token_acc = Account::unpack(&rewards_token_ai.try_borrow_data()?)?;

        if staking_token_acc.owner != staking_token_pda {
            msg!("Stake token account must have pda as owner. Current owner {}, pda {}", staking_token_acc.owner, staking_token_pda);
            return Err(ProgramError::InvalidAccountData);
        }

        if rewards_token_acc.owner != rewards_token_pda {
            msg!("Rewards token account must have pda as owner. Current owner {}, pda {}", rewards_token_acc.owner, rewards_token_pda);
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(())
    }

    fn stake(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let user_ai = next_account_info(accounts_iter)?;
        let user_staking_token_ai = next_account_info(accounts_iter)?;
        let escrow_staking_token_ai = next_account_info(accounts_iter)?;
        let user_state_ai = next_account_info(accounts_iter)?;
        let metadata_ai = next_account_info(accounts_iter)?;
        let staking_token_mint_ai = next_account_info(accounts_iter)?;
        let token_program = next_account_info(accounts_iter)?;
        let system_program = next_account_info(accounts_iter)?;
        
        let (escrow_staking_token_owner_pda, _nonce) = PdaHelper::find_staking_token_pda(staking_token_mint_ai, program_id);
        let escrow_staking_token = Account::unpack(&escrow_staking_token_ai.try_borrow_data()?)?;

        if escrow_staking_token.owner != escrow_staking_token_owner_pda {
            msg!("Stake token account must have pda as owner. Current owner {}, pda {}", escrow_staking_token.owner, escrow_staking_token_owner_pda);
            return Err(ProgramError::InvalidAccountData);
        }

        let transafer_ix = spl_token::instruction::transfer(
            &token_program.key, 
            &user_staking_token_ai.key,
            &escrow_staking_token_ai.key, 
            &user_ai.key, 
            &[], 
            amount)?;

        invoke(
            &transafer_ix, 
            &[
                user_staking_token_ai.clone(),
                escrow_staking_token_ai.clone(),
                user_ai.clone(),
                token_program.clone()
            ])?;

        msg!("Tokens transfered from staker {} to pda {}", user_staking_token_ai.key, escrow_staking_token_ai.key);

        let (user_state_pda, bump_seed) = PdaHelper::find_user_state_pda(metadata_ai, user_ai, program_id);

        msg!("Staker pda: {}", user_state_ai.key);
        msg!("Staker pda computed: {}", user_state_pda);

        if user_state_ai.try_data_is_empty()? {
            let create_acc_ix = solana_program::system_instruction::create_account(
                &user_ai.key, 
                &user_state_pda, 
                Rent::get()?.minimum_balance(UserStakingState::LEN),
                UserStakingState::LEN as u64, 
                &program_id);
    
            invoke_signed(
                &create_acc_ix,
                &[
                    user_ai.clone(),
                    user_state_ai.clone(),
                    system_program.clone()
                ], 
                &[
                    &[&metadata_ai.key.to_bytes(), &user_ai.key.to_bytes(), b"user-state", &[bump_seed]]
                ]
            )?;
        }
        
        Self::update_rewards(metadata_ai, user_state_ai)?;

        let mut user_state = UserStakingState::try_from_slice(&user_state_ai.try_borrow_data()?)?;
        
        user_state.balance += amount;

        user_state.pack(&mut user_state_ai.try_borrow_mut_data()?);

        msg!("Updated staker data at {}", user_state_ai.key);
        
        let mut metadata = StakingState::unpack(&mut metadata_ai.try_borrow_mut_data()?);
        metadata.total_supply += amount;
        metadata.pack(&mut metadata_ai.try_borrow_mut_data()?);

        msg!("Updated staking metadata at {}", metadata_ai.key);

        msg!("Account {}, staked {} tokens", user_ai.key, amount);

        Ok(())
    }

    fn unstake(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let user_ai = next_account_info(accounts_iter)?;
        let user_staking_token_ai = next_account_info(accounts_iter)?;
        let user_state_ai = next_account_info(accounts_iter)?;
        let metadata_ai = next_account_info(accounts_iter)?;
        let escrow_staking_token_ai = next_account_info(accounts_iter)?;
        let escrow_staking_token_owner_ai = next_account_info(accounts_iter)?;
        let staking_token_mint_ai = next_account_info(accounts_iter)?;
        let token_program_ai = next_account_info(accounts_iter)?;

        if !user_ai.is_signer {
            return Err(ProgramError::MissingRequiredSignature)
        }
        
        Self::update_rewards(metadata_ai, user_state_ai)?;

        let users_state = UserStakingState::unpack(&mut user_state_ai.try_borrow_mut_data()?);

        if amount > users_state.balance {
            msg!("Cannot unstake more than staked. Staked: {}, trying to withdraw: {}", users_state.balance, amount);
            return Err(ProgramError::InvalidInstructionData);
        }

        let (escrow_staking_token_owner, bump) = PdaHelper::find_staking_token_pda(staking_token_mint_ai, program_id);
        let escrow_staking_token = Account::unpack_from_slice(&mut escrow_staking_token_ai.try_borrow_mut_data()?)?;
        
        if escrow_staking_token.owner != escrow_staking_token_owner {
            msg!("Passed escrow staking owner: {}", escrow_staking_token_owner_ai.key);
            msg!("Computed escrow staking owner: {}", escrow_staking_token_owner);
            return Err(ProgramError::InvalidAccountData);
        }
        
        let transfer_ix = spl_token::instruction::transfer(
            token_program_ai.key, 
            escrow_staking_token_ai.key, 
            user_staking_token_ai.key, 
            &escrow_staking_token_owner, 
            &[], 
            amount)?;

        invoke_signed(
            &transfer_ix, 
            &[
                escrow_staking_token_ai.clone(),
                user_staking_token_ai.clone(),
                escrow_staking_token_owner_ai.clone(),
                token_program_ai.clone()
            ],
            &[
                &[&staking_token_mint_ai.key.to_bytes(), b"staking-token", &[bump]]
            ])?;

        msg!("Transfer {} tokens from staking {} account to users {} account", amount, escrow_staking_token_ai.key, user_staking_token_ai.key);

        let mut user_state = UserStakingState::unpack(&mut user_state_ai.try_borrow_mut_data()?);
        user_state.balance -= amount;
        user_state.pack(&mut user_state_ai.try_borrow_mut_data()?);

        msg!("Staker state updated: {}", user_state_ai.key);

        let mut metadata = StakingState::unpack(&mut metadata_ai.try_borrow_mut_data()?);
        metadata.total_supply -= amount;
        metadata.pack(&mut metadata_ai.try_borrow_mut_data()?);

        msg!("Staking state updated: {}", user_state_ai.key);

        Ok(())
    }

    fn get_rewards(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let user_ai = next_account_info(accounts_iter)?;
        let user_rewards_token_ai = next_account_info(accounts_iter)?;
        let user_state_ai = next_account_info(accounts_iter)?;
        let metadata_ai = next_account_info(accounts_iter)?;
        let escrow_rewards_token_ai = next_account_info(accounts_iter)?;
        let escrow_rewards_token_owner_ai = next_account_info(accounts_iter)?;
        let rewards_token_mint_ai = next_account_info(accounts_iter)?;
        let token_program_ai = next_account_info(accounts_iter)?;

        if !user_ai.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Self::update_rewards(metadata_ai, user_state_ai)?;

        let rewards = Self::get_user_rewards(user_state_ai);

        let (escrow_rewards_token_owner, bump_seed) = PdaHelper::find_rewards_token_pda(rewards_token_mint_ai, program_id);

        let transfer_ix = spl_token::instruction::transfer(
            token_program_ai.key, 
            escrow_rewards_token_ai.key, 
            user_rewards_token_ai.key, 
            &escrow_rewards_token_owner, 
            &[], 
            rewards)?;

        invoke_signed(
            &transfer_ix, 
            &[
                escrow_rewards_token_ai.clone(),
                user_rewards_token_ai.clone(),
                escrow_rewards_token_owner_ai.clone(),
                token_program_ai.clone()
            ],
            &[
                &[&rewards_token_mint_ai.key.to_bytes(), b"rewards-token", &[bump_seed]]
            ])?;

        let mut user_state = UserStakingState::unpack(&mut user_state_ai.try_borrow_mut_data()?);
        user_state.rewards = 0;
        user_state.pack(&mut user_state_ai.try_borrow_mut_data()?);

        Ok(())
    }

    fn update_rewards(state_ai: &AccountInfo, user_state_ai: &AccountInfo) -> ProgramResult {
        let mut state = StakingState::unpack(&mut state_ai.try_borrow_mut_data().unwrap());
        let mut user_state = UserStakingState::unpack(&mut user_state_ai.try_borrow_mut_data().unwrap());

        let rewards_per_token_stored = Self::reward_per_token(&state);
        let last_update_timestamp = Clock::get().unwrap().unix_timestamp;

        state.reward_per_token_stored = rewards_per_token_stored;
        state.last_update_timestamp = last_update_timestamp;

        let new_rewards = Self::earned(&state, &user_state);
        user_state.rewards = new_rewards;

        state.pack(&mut state_ai.try_borrow_mut_data()?);
        user_state.pack(&mut user_state_ai.try_borrow_mut_data()?);

        Ok(())
    }

    fn get_user_rewards(user_state_ai: &AccountInfo) -> u64 {
        let user_state = UserStakingState::unpack(&mut user_state_ai.try_borrow_mut_data().unwrap());
        return user_state.rewards;
    }

    fn reward_per_token(state: &StakingState) -> u64 {
        return Self::calc_reward_per_token(state.total_supply, state.reward_per_token_stored, state.last_update_timestamp);
    }

    fn calc_reward_per_token(total_supply: u64, rewards_per_token_stored: u64, last_update_timestamp: UnixTimestamp) -> u64 {
        let current_timestamp = Clock::get().unwrap().unix_timestamp; 
        
        if total_supply == 0 {
            return 0;
        }

        let rewards_per_token = rewards_per_token_stored + (((current_timestamp - last_update_timestamp) as u64) * Self::REWARD_RATE) / total_supply;
        
        rewards_per_token
    }

    fn earned(state: &StakingState, user_state: &UserStakingState) -> u64 {
        return Self::calc_earned(
            user_state.balance, 
            state.total_supply, 
            state.reward_per_token_stored, 
            state.last_update_timestamp, 
            user_state.reward_per_token_paid, 
            user_state.rewards);
    }

    fn calc_earned(
        stake_amount: u64, 
        total_supply: u64, 
        rewards_per_token_stored: u64, 
        last_update_timestamp: UnixTimestamp, 
        user_reward_per_token_paid: u64, 
        user_rewards: u64) -> u64 {
        let rewards_per_token = Self::calc_reward_per_token(total_supply, rewards_per_token_stored, last_update_timestamp);
        let earned = stake_amount * ( rewards_per_token - user_reward_per_token_paid) + user_rewards;

        earned
    }
}