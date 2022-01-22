use crate::{
    processor::Processor
};
use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    Processor::process_instruction(program_id, accounts, instruction_data)?;
    Ok(())
}

entrypoint!(process_instruction);