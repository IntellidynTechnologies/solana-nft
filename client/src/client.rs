use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{ RpcAccountInfoConfig, RpcProgramAccountsConfig },
    rpc_filter::{ RpcFilterType, MemcmpEncodedBytes, Memcmp },
};

use solana_sdk::{
    commitment_config::{ CommitmentConfig, CommitmentLevel },
    signer::{
        keypair::Keypair,
        Signer,
    },
    signature::Signature,
    transaction::Transaction,
    borsh::try_from_slice_unchecked,
    program_pack::Pack
};
use solana_program::pubkey::Pubkey;

use solana_account_decoder::UiAccountEncoding;

use alloy_token_program::{
    instruction::NftInstruction,
    state::{AlloyData, PREFIX},
};

use spl_token::state::Mint;

use crate::cl_errors::CustomError;

pub struct NftClient {
    pub client: RpcClient
}

impl NftClient {
    pub fn new() -> Result<Self, CustomError> {
        
        let url = "https://api.devnet.solana.com".to_string();
        let commitment_config = CommitmentConfig::confirmed();

        Ok(NftClient {
            client: RpcClient::new_with_commitment(
                url,
                commitment_config,
            )
        })
    }

    pub fn get_balance(&self, keypair: &Keypair) -> u64 {
        let balance = self.client.get_balance(&keypair.pubkey()).unwrap();
        println!("---> Balance: {}", &balance);

        balance
    }

    pub fn airdrop(&self, to_pubkey: &Pubkey, lamports: u64) -> Result<Signature, CustomError> {
        let blockhash = self.client.get_latest_blockhash()?;
        let signature = self.client.request_airdrop_with_blockhash(to_pubkey, lamports, &blockhash)?;
        self.client.confirm_transaction_with_spinner(&signature, &blockhash, self.client.commitment())?;

        Ok(signature)
    }

    pub fn create_alloy_data_accounts(
        &self,
        payer: &Keypair,
        name: String,
        uri: String,
        last_price: u64,
        listed_price: u64,
        owner: &Keypair,
        lamports: Option<u64>,
    ) -> Result<(AlloyData, Pubkey), CustomError> {
        let program_key = alloy_token_program::id();
        println!("---> Program ID: {}\n", program_key);

        let accounts = self.client.get_program_accounts(&program_key).unwrap();
        println!("---> Saved alloy accounts: {}", accounts.len());
        let id = accounts.len() as u8 + 1;

        let lamports = if let Some(lamports) = lamports {
            lamports
        } else {
            self.client.get_minimum_balance_for_rent_exemption(Mint::LEN)?
        };

        let alloy_data_seeds = &[PREFIX.as_bytes(), &program_key.as_ref(),&[id]];
        let (alloy_data_key, _) = Pubkey::find_program_address(alloy_data_seeds, &program_key);
        println!("---> Alloy Data Key: {:?}", &alloy_data_key);

        println!("--->\n Id: {},\n Name: {},\n Uri: {},\n Last_price: {},\n Listed_price: {},\n Owner: {}\n",
            id, name, uri, last_price, listed_price, &owner.pubkey()
        );

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
            &[payer],
            latest_blockhash
        );

        let result = self.client.send_and_confirm_transaction_with_spinner_and_commitment(&transaction, CommitmentConfig::confirmed());
        println!("---> Result: {:#?}", &result);
        if result.is_ok() {
            println!(
                "Successfully created a Mint Account with Pubkey: {:?}",
                payer
            )
        };
    
        let account_data = self.client.get_account_data(&alloy_data_key);
        
        if !&account_data.is_err() && &account_data.as_ref().unwrap().len() != &0 {
            let alloy_data: AlloyData = try_from_slice_unchecked(&account_data.unwrap()).unwrap();

            println!(
                "Create alloy data account with owner {:?} and key {:?} and name of {:?} and id of {}",
                &alloy_data.owner_address, &alloy_data_key, &alloy_data.name, &alloy_data.id
            );

            Ok((alloy_data, alloy_data_key))
        } else {
            return Err(CustomError::InvalidInput.into());
        }
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
        println!("{:#?}", &all_alloys);
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
        
        let filter1 = Memcmp{
            offset: 0,
            bytes: MemcmpEncodedBytes::Base58(alloy_data.owner_address.to_string()),
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
            filters: Some(vec![solana_client::rpc_filter::RpcFilterType::Memcmp(filter1), filter2]),
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
            &alloy_data.owner_address,
        );
    
        let latest_blockhash = self.client.get_latest_blockhash().unwrap();

        let transaction = Transaction::new_signed_with_payer(
            &[new_alloy_data_instruction],
            Some(&payer.pubkey()),
            &vec![payer],
            latest_blockhash,
        );

        let _ = self.client.send_and_confirm_transaction_with_spinner(&transaction);

        let account = self.client.get_account(&alloy_data_key).unwrap();
        let alloy_data: AlloyData = try_from_slice_unchecked(&account.data).unwrap();
        println!("Purchased Alloy Data: name-{} price-{} owner-{}", alloy_data.name, alloy_data.listed_price, alloy_data.owner_address);
        (alloy_data, alloy_data_key)
    }
}