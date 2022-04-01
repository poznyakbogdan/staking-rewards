use solana_program::{
    pubkey::Pubkey,
    sysvar::slot_history::AccountInfo
};

pub struct PdaHelper;

impl PdaHelper {

    pub fn find_metadata_pda(staking_token_mint_ai: &AccountInfo, rewards_token_mint_ai: &AccountInfo, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[&staking_token_mint_ai.key.to_bytes(), &rewards_token_mint_ai.key.to_bytes(), b"metadata"], 
            program_id)
    }

    pub fn find_staking_token_pda(staking_token_mint_ai: &AccountInfo, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[&staking_token_mint_ai.key.to_bytes(), b"staking-token"], 
            program_id)
    }

    pub fn find_rewards_token_pda(rewards_token_mint_ai: &AccountInfo, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[&rewards_token_mint_ai.key.to_bytes(), b"rewards-token"], 
            program_id)
    }

    pub fn find_user_state_pda(metadata_ai: &AccountInfo, user_ai: &AccountInfo, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[&metadata_ai.key.to_bytes(), &user_ai.key.to_bytes(), b"user-state"], 
            program_id)
    }
}