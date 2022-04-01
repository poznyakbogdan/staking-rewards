use crate::processor::Processor;
use solana_program::{
    pubkey::Pubkey,
    entrypoint,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
};

entrypoint!(process_entrypoint);
fn process_entrypoint(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    Processor::process(program_id, accounts, instruction_data)
}