use solana_client::{
    client_error::ClientError,
    rpc_client::RpcClient,
    rpc_config::{ RpcAccountInfoConfig, RpcProgramAccountsConfig },
    rpc_filter::{ RpcFilterType, MemcmpEncodedBytes, RpcFilterType::Memcmp },
};

use solana_sdk::{
    commitment_config::CommitmentConfig,
    message::Message,
    signer::{
        keypair::{read_keypair_file, write_keypair_file, Keypair},
        Signer,
    },
    transaction::Transaction,
    {borsh::try_from_slice_unchecked, program_pack::Pack},
};
use solana_program::{
    pubkey::Pubkey,
    instruction::Instruction,
};

use solana_account_decoder::{
    parse_account_data::{parse_account_data, AccountAdditionalData, ParsedAccount},
    UiAccountEncoding,
};

use alloy_token_program::{
    instruction::NftInstruction,
    state::{AlloyData, PREFIX},
};

use crate::cl_errors::CustomError;
use spl_token::state::{ Account, Mint };

pub struct Client {
    pub client: RpcClient
}

impl Client {
    pub fn new() -> Result<Self, CustomError> {
        
        let url = "http://localhost:8899/".to_string();
        let commitment_config = CommitmentConfig::confirmed();

        Ok(Client {
            client: RpcClient::new_with_commitment(
                url,
                commitment_config,
            )
        })
    }

    pub fn create_alloy_data_accounts(
        &self,
        payer: &Keypair,
        name: String,
        uri: String,
        last_price: u64,
        listed_price: u64,
        owner: &Keypair 
    ) -> (AlloyData, Pubkey) {
        let mint_account = Keypair::new();

        let program_key = alloy_token_program::id();
        println!("---> Program ID: {}\n", program_key);

        let accounts = self.client.get_program_accounts(&program_key).unwrap();
        println!("---> Saved alloy accounts: {}", accounts.len());
        let id = accounts.len() as u8 + 1;

        let alloy_data_seeds = &[PREFIX.as_bytes(), &program_key.as_ref(),&[id]];
        let (alloy_data_key, _) = Pubkey::find_program_address(alloy_data_seeds, &program_key);

        let new_alloy_data_instruction = NftInstruction::create_alloy_data_accounts(
            &program_key,
            &alloy_data_key,
            &payer.pubkey(),
            id,
            name,
            uri,
            last_price,
            listed_price,
            &owner.pubkey()
        );

        let latest_blockhash = self.client.get_latest_blockhash().unwrap();

        let transaction: Transaction = Transaction::new_signed_with_payer(
            &[new_alloy_data_instruction],
            Some(&payer.pubkey()),
            &[&mint_account, &payer],
            latest_blockhash
        );

        let result = self.client.send_and_confirm_transaction_with_spinner(&transaction);

        if result.is_ok() {
            println!(
                "Successfully created a Mint Account with Pubkey: {:?}",
                mint_account
            )
        };
    
        let account = self.client.get_account(&alloy_data_key).unwrap();
        let alloy_data: AlloyData = try_from_slice_unchecked(&account.data).unwrap();

        (alloy_data, alloy_data_key)
    }

    pub fn update_alloy_data_account(
        &self,
        payer: &Keypair,
        id: u8,
        listed_price: u64,
        owner: &Keypair
    ) -> (AlloyData, Pubkey) {
        let mint_account = Keypair::new();

        let program_key = alloy_token_program::id();
        println!("---> Program ID: {}\n", program_key);

        let alloy_data_seeds = &[PREFIX.as_bytes(), &program_key.as_ref(),&[id]];
        let (alloy_data_key, _) = Pubkey::find_program_address(alloy_data_seeds, &program_key);

        let new_alloy_data_instruction = NftInstruction::update_alloy_price(
            &program_key,
            &alloy_data_key,
            id,
            listed_price.try_into().unwrap(),
            &payer.pubkey(),
            &owner.pubkey()
        );

        let latest_blockhash = self.client.get_latest_blockhash().unwrap();

        let transaction: Transaction = Transaction::new_signed_with_payer(
            &[new_alloy_data_instruction],
            Some(&payer.pubkey()),
            &[owner, payer],
            latest_blockhash
        );

        let result = self.client.send_and_confirm_transaction_with_spinner(&transaction);

        if result.is_ok() {
            println!(
                "Successfully updated the Alloy Data with Pubkey: {:?}",
                mint_account
            )
        };
    
        let account = self.client.get_account(&alloy_data_key).unwrap();
        let alloy_data: AlloyData = try_from_slice_unchecked(&account.data).unwrap();
        println!("Updated Alloy Data: name-{} new_price-{}", alloy_data.name, alloy_data.listed_price);
        (alloy_data, alloy_data_key)
    }

    pub fn get_all_alloys(&self) -> Vec<AlloyData> {
        let program_key = alloy_token_program::id();
        println!("---> Program ID: {}\n", program_key);

        let accounts = self.client.get_program_accounts(&program_key).unwrap();
        println!("--> Saved program accounts: {}", accounts.len());

        let mut all_alloys: Vec<AlloyData> = Vec::new();

        for (pubkey, account) in accounts {
            let alloy_data: AlloyData = try_from_slice_unchecked(&account.data).unwrap();
            all_alloys.push(alloy_data);
        }

        all_alloys
    }

    pub fn get_owners_all_alloys(&self, owner: &Keypair) -> Vec<AlloyData> {
        let program_key = alloy_token_program::id();
        println!("---> Program ID: {}\n", program_key);

        let accounts = self.client.get_program_accounts(&program_key).unwrap();
        println!("--> Saved program accounts: {}", accounts.len());

        let mut all_alloys: Vec<AlloyData> = Vec::new();

        for (pubkey, account) in accounts {

            if owner.pubkey() == account.owner {
                let alloy_data: AlloyData = try_from_slice_unchecked(&account.data).unwrap();
                all_alloys.push(alloy_data);
            }
        }

        all_alloys
    }

    pub fn purchase_alloy(
        &self,
        payer: &Keypair,
        id: u8,
        new_name: Option<String>,
        new_uri: Option<String>,
        new_price: Option<u64>,
    ) -> (AlloyData, Pubkey) {
        let program_key = alloy_token_program::id();
        
        let alloy_data_seeds = &[PREFIX.as_bytes(), &program_key.as_ref(),&[id]];
        let (alloy_data_key, _) = Pubkey::find_program_address(alloy_data_seeds, &program_key);
        
        let account = self.client.get_account(&alloy_data_key).unwrap();
        let alloy_data: AlloyData = try_from_slice_unchecked(&account.data).unwrap();
        
        let filter1 = RpcFilterType::Memcmp {
            offset: 0,
            bytes: MemcmpEncodedBytes::Binary(alloy_data.owner_address.to_string()),
            encoding: None,
        };
        let filter2 = RpcFilterType::DataSize(165);
        let account_config = RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: None,
            commitment: Some(CommitmentConfig {
                commitment: CommitmentLevel::Confirmed,
            }),
        };
    
        let config = RpcProgramAccountsConfig {
            filters: Some(vec![filter1, filter2]),
            account_config,
            with_context: None,
        };
    
        let holders = self.client.get_program_accounts_with_config(&spl_token::id(), config).unwrap();
    
        println!("holder {}", holders[0].0.to_string());
    
        let new_alloy_data_instruction = NftInstruction::purchase_alloy(
            &program_key,
            &alloy_data_key,
            id,
            new_name,
            new_uri,
            new_price,
            &payer.pubkey(),
            alloy_data.owner_address,
        );
    
        let latest_blockhash = self.client.get_latest_blockhash().unwrap();

        let transaction = Transaction::new_signed_with_payer(
            &[new_alloy_data_instruction],
            Some(&payer.pubkey()),
            &vec![payer],
            latest_blockhash,
        );

        let account = self.client.get_account(&alloy_data_key).unwrap();
        let alloy_data: AlloyData = try_from_slice_unchecked(&account.data).unwrap();
        println!("Purchased Alloy Data: name-{} price-{} owner-{}", alloy_data.name, alloy_data.listed_price, alloy_data.owner_address);
        (alloy_data, alloy_data_key)
    }
}