
use solana_program::pubkey::Pubkey;
use solana_program::clock::UnixTimestamp;
use borsh::{BorshSerialize, BorshDeserialize};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct StakingState {
    pub admin: Pubkey,
    pub staking_token_mint: Pubkey,
    pub reward_token_mint: Pubkey,
    pub total_supply: u64, 
    pub reward_per_token_stored: u64,
    pub last_update_timestamp: UnixTimestamp,
}

impl StakingState {
    pub const LEN: usize = 32 * 3 + 8 * 3;

    pub fn unpack(data: &mut [u8]) -> Self {
        StakingState::try_from_slice(data).unwrap()
    }

    pub fn pack(&self, data: &mut [u8]) {
        let encoded = self.try_to_vec().unwrap();
        data[..encoded.len()].copy_from_slice(&encoded);
    }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct UserStakingState {
    pub balance: u64, 
    pub reward_per_token_paid: u64,
    pub rewards: u64,
}

impl UserStakingState {
    pub const LEN: usize = 8 * 3;

    pub fn unpack(data: &mut [u8]) -> Self {
        UserStakingState::try_from_slice(data).unwrap()
    }

    pub fn pack(&self, data: &mut [u8]) {
        let encoded = self.try_to_vec().unwrap();
        data[..encoded.len()].copy_from_slice(&encoded);
    }
}