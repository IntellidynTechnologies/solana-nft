use crate::{
	state::{ AlloyData, PREFIX, MAX_DATA_SIZE, MAX_NAME_LENGTH, MAX_URI_LENGTH },
	instruction::NftInstruction,
	error::CustomError,
};
use borsh::{ BorshSerialize, BorshDeserialize };
use solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program_error::ProgramError,
        pubkey::Pubkey,
        program::{ invoke, invoke_signed },
        sysvar::rent::Rent,
        sysvar::Sysvar,
        system_instruction,
};
use spl_token::state::{Account, Mint};

pub struct Processor;

impl Processor {
	pub fn process_instruction(
		program_id: &Pubkey,
		accounts: &[AccountInfo],
		instruction_data: &[u8]
	) -> ProgramResult {
		let instruction = NftInstruction::try_from_slice(instruction_data)?;

		match instruction {
			NftInstruction::CreateAlloyDataAccount(args) => {
				msg!("Instruction: Create Alloy Data Accounts");
				process_create_alloy_data_accounts(
					program_id,
			                accounts,
			                args.data,
			                args.id,
			        )
			}
			// NftInstruction::UpdateAlloyPrice(args) => {
			//         msg!("Instruction: Update Alloy Price from Id");
			//         process_update_alloy_price(
			//                 program_id,
			//                 accounts,
			//                 args.id,
			//                 args.price,
			//         )
	  //       	},
		 //        NftInstruction::PurchaseAlloy(args) => {
		 //        	msg!("Instruction: Purchase Alloy from Id");
		 //            	process_purchase_alloy(
		 //                	program_id,
		 //                	accounts,
		 //                	args.id,
		 //                	args.new_name,
		 //                	args.new_uri,
		 //                	args.new_price,
		 //            	)
		 //        }

		}
	}
}

pub fn process_create_alloy_data_accounts(
	program_id: &Pubkey,
	accounts: &[AccountInfo],
	data: AlloyData,
	id: u8
) -> ProgramResult {
	let account_iter = &mut accounts.iter();

	let alloy_data_account_info = next_account_info(account_iter)?;
	let payer_info = next_account_info(account_iter)?;
	let system_account_info = next_account_info(account_iter)?;
	let rent_account_info = next_account_info(account_iter)?;

	let alloy_data_seeds = &[
		PREFIX.as_bytes(),
		program_id.as_ref(),
		&[id]
	];

	let (alloy_data_key, alloy_data_bump_seed) = Pubkey::find_program_address(alloy_data_seeds, program_id);
	let alloy_data_authority_signer_seeds = &[
		PREFIX.as_bytes(),
        	program_id.as_ref(),
        	&[id],
        	&[alloy_data_bump_seed],
    	];

    	if *alloy_data_account_info.key != alloy_data_key {
    		return Err(CustomError::InvalidAlloyDataKey.into());
    	}

    	let rent = &Rent::from_account_info(rent_account_info)?;
    	let req_lamports = rent.minimum_balance(MAX_DATA_SIZE).max(1).saturating_sub(alloy_data_account_info.lamports());

    	if req_lamports > 0 {
    		msg!("{} lamports are transferred to the new acccount", req_lamports);
    		invoke(
    			&system_instruction::transfer(&payer_info.key, alloy_data_account_info.key, req_lamports),
    			&[
    			payer_info.clone(),
    			alloy_data_account_info.clone(),
    			system_account_info.clone()
    			],
    		)?;
    	}

    	let accounts = &[alloy_data_account_info.clone(), system_account_info.clone()];

    	msg!("Allocate space for the account.");
    	invoke_signed(
    		&system_instruction::allocate(alloy_data_account_info.key, MAX_DATA_SIZE.try_into().unwrap()),
    		accounts,
    		&[alloy_data_authority_signer_seeds],
    	)?;

    	let mut alloy_data = AlloyData::from_acc_info(alloy_data_account_info)?;

    	if data.name.len() > MAX_NAME_LENGTH {
    		return Err(CustomError::NameTooLong.into());
    	}

    	if data.uri.len() > MAX_URI_LENGTH {
    		return Err(CustomError::UriTooLong.into());
    	}

    	alloy_data.id = data.id;
    	alloy_data.name = data.name;
    	alloy_data.uri = data.uri;
    	alloy_data.last_price = data.last_price;
    	alloy_data.listed_price = data.listed_price;
    	alloy_data.owner_address = data.owner_address;

    	//puff out data fields
    	let mut array_of_zeroes = vec![];

    	while array_of_zeroes.len() > MAX_NAME_LENGTH - alloy_data.name.len() {
    		array_of_zeroes.push(0u8);
    	}

    	alloy_data.name = alloy_data.name.clone() + std::str::from_utf8(&array_of_zeroes).unwrap();

    	let mut array_of_zeroes = vec![];

    	while array_of_zeroes.len() > MAX_URI_LENGTH - alloy_data.uri.len() {
    		array_of_zeroes.push(0u8);
    	}

    	alloy_data.uri = alloy_data.uri.clone() + std::str::from_utf8(&array_of_zeroes).unwrap();

    	alloy_data.serialize(&mut *alloy_data_account_info.data.borrow_mut())?;
    	msg!("Alloy Data Saved!");

	Ok(())
}

// pub fn process_create_alloy_data_accounts(
// 	program_id: &Pubkey,
// 	accounts: &[AccountInfo],
// 	data: AlloyData,
// 	id: u8
// ) -> ProgramResult {
// 	let account_iter = &mut accounts.iter();

// 	let alloy_data_account = next_account_info(account_iter)?;
// 	let payer = next_account_info(account_iter)?;
// 	let system_account = next_account_info(account_iter)?;
// 	let rent_account = next_account_info(account_iter)?;

// 	Ok(())
// }

// pub fn process_create_alloy_data_accounts(
// 	program_id: &Pubkey,
// 	accounts: &[AccountInfo],
// 	data: AlloyData,
// 	id: u8
// ) -> ProgramResult {
// 	let account_iter = &mut accounts.iter();

// 	let alloy_data_account = next_account_info(account_iter)?;
// 	let payer = next_account_info(account_iter)?;
// 	let system_account = next_account_info(account_iter)?;
// 	let rent_account = next_account_info(account_iter)?;

// 	Ok(())
// }