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
        program_pack::{IsInitialized, Pack},
};
use spl_token::state::Account;

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
			        )
			},
			NftInstruction::UpdateAlloyPrice(args) => {
				msg!("Instruction: Update Alloy Price from Id");
				process_update_alloy_price(
						program_id,
						accounts,
						args.id,
						args.price,
				)
			},
			NftInstruction::PurchaseAlloy(args) => {
				msg!("Instruction: Purchase Alloy from Id");
				process_purchase_alloy(
					program_id,
					accounts,
					args.id,
					args.new_name,
					args.new_uri,
					args.new_price,
				)
			}

		}
	}
}

pub fn process_create_alloy_data_accounts(
	program_id: &Pubkey,
	accounts: &[AccountInfo],
	data: AlloyData,
) -> ProgramResult {
	let account_iter = &mut accounts.iter();

	let alloy_data_account_info = next_account_info(account_iter)?;
	let payer_info = next_account_info(account_iter)?;
	let system_account_info = next_account_info(account_iter)?;
	let rent_account_info = next_account_info(account_iter)?;

	let alloy_data_seeds = &[
		PREFIX.as_bytes(),
		program_id.as_ref(),
		&[data.id]
	];

	let (alloy_data_key, alloy_data_bump_seed) = Pubkey::find_program_address(alloy_data_seeds, program_id);
	msg!("Alloy Data Key: {:?}", &alloy_data_key);

	let alloy_data_authority_signer_seeds = &[
		PREFIX.as_bytes(),
        	program_id.as_ref(),
        	&[data.id],
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
    	msg!("Alloy Data Saved! {:#?}", alloy_data);

	Ok(())
}

pub fn process_update_alloy_price(
	program_id: &Pubkey,
	accounts: &[AccountInfo],
	alloy_id: u8,
	new_price: u64
) -> ProgramResult {
	let account_iter = &mut accounts.iter();

	let alloy_data_account_info = next_account_info(account_iter)?;
	let owner_info = next_account_info(account_iter)?;
	let owner_nft_account_info = next_account_info(account_iter)?;

	let alloy_data_seeds = &[
		PREFIX.as_bytes(),
		program_id.as_ref(),
		&[alloy_id]
	];

	let (alloy_data_key, _alloy_data_bump_seed) = Pubkey::find_program_address(alloy_data_seeds, program_id);

	if *alloy_data_account_info.key != alloy_data_key {
		return Err(CustomError::InvalidAlloyDataKey.into());
	}

	if alloy_data_account_info.owner != program_id {
		return Err(CustomError::IncorrectOwner.into());
	}

	let mut alloy_data = AlloyData::from_acc_info(alloy_data_account_info)?;

	let token_acc: Account = assert_initialized(&owner_nft_account_info)?;

	if owner_nft_account_info.owner != &spl_token::id() {
		return Err(CustomError::IncorrectOwner.into());
	};

	if alloy_data.owner_address != token_acc.mint {
		return Err(CustomError::OwnerMismatch.into());
	}

	if token_acc.owner != *owner_info.key {
        	return Err(CustomError::InvalidOwner.into());
    	}

    	alloy_data.listed_price = new_price;

    	alloy_data.serialize(&mut *alloy_data_account_info.data.borrow_mut())?;

	Ok(())
}

pub fn process_purchase_alloy(
	program_id: &Pubkey,
	accounts: &[AccountInfo],
	id: u8,
	new_name: Option<String>,
	new_uri: Option<String>,
	new_price: Option<u64>
) -> ProgramResult {
	let account_iter = &mut accounts.iter();

	let alloy_data_account_info = next_account_info(account_iter)?;
	let payer_info = next_account_info(account_iter)?;
	let nft_owner_address_info = next_account_info(account_iter)?;
	let nft_token_account_info = next_account_info(account_iter)?;
	let system_account_info = next_account_info(account_iter)?;

	let alloy_data_seeds = &[
		PREFIX.as_bytes(),
		program_id.as_ref(),
		&[id]
	];

	let (alloy_data_key, _alloy_data_bump_seed) = Pubkey::find_program_address(alloy_data_seeds, program_id);

	if *alloy_data_account_info.key != alloy_data_key {
		return Err(CustomError::InvalidAlloyDataKey.into());
	}

	let mut alloy_data = AlloyData::from_acc_info(alloy_data_account_info)?;
	let token_acc: Account = assert_initialized(&nft_token_account_info)?;

	if *nft_owner_address_info.key != token_acc.owner {
		return Err(CustomError::OwnerMismatch.into());
	}

	if alloy_data.owner_address != token_acc.owner {
		return Err(CustomError::InvalidOwner.into());
	}

	invoke(
        &system_instruction::transfer(&payer_info.key, &nft_owner_address_info.key, alloy_data.listed_price as u64),
        &[
            payer_info.clone(),
            nft_owner_address_info.clone(),
            system_account_info.clone(),
        ],
    )?;

	alloy_data.name = match new_name {
		Some(new_name) => new_name,
		None => alloy_data.name
	};

	alloy_data.uri = match new_uri {
		Some(new_uri) => new_uri,
		None => alloy_data.uri
	};

	alloy_data.last_price = alloy_data.listed_price;
    alloy_data.listed_price = match new_price {
        Some(price) => {
            price
        }
        None => {
            alloy_data.listed_price
        }
    };

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
	msg!("Alloy Data Replaced!");

	Ok(())
}

pub fn assert_initialized<T: Pack + IsInitialized>(
	account_info: &AccountInfo,
) -> Result<T, ProgramError> {
	let account: T = T::unpack_unchecked(&account_info.data.borrow())?;
    	if !account.is_initialized() {
        	Err(CustomError::Uninitialized.into())
    	} else {
        	Ok(account)
    	}
}