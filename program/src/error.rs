use solana_program::program_error::ProgramError;

pub enum NftError {
    SignerNotOwnerOrApproval,
    Overflow,
}

impl From<NftError> for ProgramError {
    fn from(e: NftError) -> Self {
        ProgramError::Custom(e as u32)
    }
}