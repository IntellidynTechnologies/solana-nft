use solana_program::{
	pubkey::Pubkey,
	program_error::ProgramError,
	account_info::AccountInfo,
	borsh as sol_borsh
};
use crate::error::CustomError;

use borsh::{ BorshSerialize, BorshDeserialize };

pub const PREFIX: &str = "alloy";
pub const MAX_NAME_LENGTH: usize = 32;
pub const MAX_SYMBOL_LENGTH: usize = 10;
pub const MAX_URI_LENGTH: usize = 200;
pub const MAX_DATA_SIZE: usize = 1 + 4 + MAX_NAME_LENGTH + 4 + MAX_URI_LENGTH + 8 + 8 +32;

#[derive(Debug, BorshSerialize, BorshDeserialize, PartialEq, Clone)]
pub struct AlloyData {
	pub id: u8,
	pub name: String,
	pub symbol: String,
	pub uri: String,
	pub last_price: u64,
	pub listed_price: u64,
	pub owner_address: Pubkey,
}

impl AlloyData {
	pub fn from_acc_info(acc_info: &AccountInfo) -> Result<Self, ProgramError> {
		
		let acc_info_data = &acc_info.data.borrow_mut();

		if acc_info_data.len() != MAX_DATA_SIZE {
			return Err(CustomError::DataTypeMismatch.into());
		}

		let result = sol_borsh::try_from_slice_unchecked(acc_info_data)?;

		Ok(result)
	}
}