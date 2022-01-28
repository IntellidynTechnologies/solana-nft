use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{ RpcAccountInfoConfig, RpcProgramAccountsConfig },
    rpc_filter::{ RpcFilterType, MemcmpEncodedBytes, Memcmp },
    rpc_request::TokenAccountsFilter,
    rpc_response::RpcKeyedAccount,
    client_error::ClientError
};

use solana_sdk::{
    pubkey::Pubkey,
    instruction::Instruction,
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

use solana_account_decoder::UiAccountEncoding;

use std::str::FromStr;

use alloy_token_program::{
    instruction::NftInstruction,
    state::{AlloyData, PREFIX},
};

use spl_token::state::{ Account, Mint };

use crate::cl_errors::CustomError;

pub type ClientResult<T> = Result<T, CustomError>;

pub const DEFAULT_LAMPORTS_PER_SOL: u64 = 1_000_000_000;

pub struct NftClient {
    pub client: RpcClient
}

impl NftClient {
    pub fn new() -> ClientResult<Self> {
        
        let url = "https://api.devnet.solana.com".to_string();
        let commitment_config = CommitmentConfig::confirmed();

        Ok(NftClient {
            client: RpcClient::new_with_commitment(
                url,
                commitment_config,
            )
        })
    }

    pub fn get_balance(&self, acc: &Pubkey) -> ClientResult<u64> {
        let balance = self.client.get_balance(acc).unwrap();
        println!("---> Balance: {}", &balance);

        Ok(balance)
    }

    pub fn airdrop(&self, to_pubkey: &Pubkey, lamports: u64) -> ClientResult<Signature> {
        let blockhash = self.client.get_latest_blockhash()?;
        let signature = self.client.request_airdrop_with_blockhash(to_pubkey, lamports, &blockhash)?;
        self.client.confirm_transaction_with_spinner(&signature, &blockhash, self.client.commitment())?;

        Ok(signature)
    }

    pub fn get_total_nfts(&self, ) -> ClientResult<usize> {
        let program_key = alloy_token_program::id();
        Ok(self.client.get_program_accounts(&program_key).unwrap().len())
    }

    pub fn create_mint_account(
        &self,
        wallet_keypair: &Keypair,
        mint_account: &Keypair,
        lamports: Option<u64>,
    ) -> ClientResult<Pubkey> {
        let mint_account_pk = mint_account.pubkey();

        let lamports = if let Some(lamports) = lamports {
            lamports
        } else {
            self.client.get_minimum_balance_for_rent_exemption(Mint::LEN)?
        };

        let create_account_instruction: Instruction = solana_sdk::system_instruction::create_account(
            &wallet_keypair.pubkey(),
            &mint_account_pk,
            lamports,
            Mint::LEN as u64,
            &spl_token::id(),
        );

        let initialize_mint_instruction: Instruction = spl_token::instruction::initialize_mint(
            &spl_token::id(),
            &mint_account_pk,
            &wallet_keypair.pubkey(),
            None,
            0
        ).unwrap();

        let latest_blockhash = self.client.get_latest_blockhash().unwrap();

        let transaction: Transaction = Transaction::new_signed_with_payer(
            &vec![create_account_instruction, initialize_mint_instruction],
            Some(&wallet_keypair.pubkey()),
            &[mint_account, wallet_keypair],
            latest_blockhash
        );

        let result = self.client.send_and_confirm_transaction_with_spinner_and_commitment(&transaction, CommitmentConfig::confirmed());
        println!("Result --> {:#?}", &result);

        if result.is_ok() {
            println!(
                "Successfully created a Mint Account with Pubkey: {:?}",
                &mint_account_pk
            )
        };

        Ok(mint_account_pk)
    }

    pub fn create_token_account(
        &self,
        wallet_keypair: &Keypair,
        acc_mint_to: &Keypair,
        mint_account_pk: &Pubkey,
    ) -> ClientResult<Pubkey> {

        let create_account_instruction: Instruction = solana_sdk::system_instruction::create_account(
            &wallet_keypair.pubkey(),
            &acc_mint_to.pubkey(),
            self.client
                .get_minimum_balance_for_rent_exemption(Account::LEN)
                .unwrap(),
            Account::LEN as u64,
            &spl_token::id(),
        );

        let initialize_account2_instruction: Instruction = spl_token::instruction::initialize_account2(
            &spl_token::id(),
            &acc_mint_to.pubkey(),
            &mint_account_pk,
            &wallet_keypair.pubkey(),
        )
        .unwrap();

        let latest_blockhash = self.client.get_latest_blockhash().unwrap();

        let transaction: Transaction = Transaction::new_signed_with_payer(
            &vec![create_account_instruction, initialize_account2_instruction],
            Some(&wallet_keypair.pubkey()),
            &[wallet_keypair, acc_mint_to],
            latest_blockhash
        );

        let result = self.client.send_and_confirm_transaction_with_spinner(&transaction);
        if result.is_ok() {
            println!(
                "Successfully created a Token Account with Pubkey: {:?}",
                acc_mint_to.pubkey()
            )
        };
    
        Ok(acc_mint_to.pubkey())

    }

    pub fn create_alloy_data_accounts(
        &self,
        name: String,
        symbol: String,
        uri: String,
        listed_price: f64,
        wallet_keypair: &Keypair,
        &mint_account_pubkey: &Pubkey,
    ) -> ClientResult<(AlloyData, Pubkey)> {
        let program_key = alloy_token_program::id();
        println!("--> Program ID: {}", &program_key);

        let accounts = self.client.get_program_accounts(&program_key).unwrap();
        println!("--> Saved alloy accounts: {}", accounts.len());

        let id = accounts.len() as u8 + 1;

        let last_price = 0 as u64;
        let listed_price = (listed_price * DEFAULT_LAMPORTS_PER_SOL as f64) as u64;
        let alloy_data_seeds = &[PREFIX.as_bytes(), &program_key.as_ref(),&[id]];
        let (alloy_data_key, _) = Pubkey::find_program_address(alloy_data_seeds, &program_key);
        println!("--> Alloy Data Key: {}", &alloy_data_key);

        let new_alloy_data_instruction = NftInstruction::create_alloy_data_accounts(
            &program_key,
            &alloy_data_key,
            &wallet_keypair.pubkey(),
            id,
            name,
            symbol,
            uri,
            last_price,
            listed_price,
            &mint_account_pubkey,
        );

        let latest_blockhash = self.client.get_latest_blockhash().unwrap();

        let transaction = Transaction::new_signed_with_payer(
            &vec![new_alloy_data_instruction], 
            Some(&wallet_keypair.pubkey()),
            &vec![wallet_keypair],
            latest_blockhash
        );

        let result = self.client.send_and_confirm_transaction_with_spinner(&transaction);
        println!("{:#?}", &result);
        if result.is_ok() {
            println!(
                "Successfully created an Alloy Data Account with Pubkey: {:?}",
                alloy_data_key
            )
        };

        let account_data = self.client.get_account_data(&alloy_data_key).unwrap();
        let alloy_data = try_from_slice_unchecked(&account_data);
        println!("Alloy Data: {:#?}", &alloy_data);
        if !alloy_data.is_err() {
            return Ok((alloy_data.unwrap(), alloy_data_key));
        } else {
            return Err(CustomError::Custom("Unxpected Length Of Input".to_string()));
        }
    }

    pub fn get_tokens_by_owner(&self, owner: &Pubkey) -> Result<Vec<RpcKeyedAccount>, ClientError> {
        let token_account_filter = TokenAccountsFilter::Mint(*owner);

        let res = self.client.get_token_accounts_by_owner(owner, token_account_filter);
        println!("--> Result: {:#?}", &res);

        res
    }

    pub fn get_all_owners_by_alloy_uri(&self, alloy_uri: String) -> ClientResult<Vec<Pubkey>> {
        let program_key = alloy_token_program::id();
        println!("---> Program ID: {}\n", program_key);

        let accounts = self.client.get_program_accounts(&program_key).unwrap();
        println!("--> Saved program accounts: {}", accounts.len());

        let mut all_owners: Vec<Pubkey> = Vec::new();

        for (_, account) in accounts {
            let alloy_data: AlloyData = try_from_slice_unchecked(&account.data).unwrap();
            if alloy_data.uri == alloy_uri {
                all_owners.push(alloy_data.owner_address);
            }
        }
        println!("--> Owners: {:#?}", &all_owners);
        Ok(all_owners)
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

        for (_, account) in accounts {
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

        for (_, account) in accounts {

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
    
        println!("--> Holder {}", holders[0].0.to_string());
    
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