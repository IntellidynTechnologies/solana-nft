use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{ RpcAccountInfoConfig, RpcProgramAccountsConfig },
    rpc_filter::{ RpcFilterType, MemcmpEncodedBytes, Memcmp },
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

use alloy_token_program::{
    instruction::NftInstruction,
    state::{AlloyData, PREFIX},
};

use spl_token::state::{ Account, Mint };

use crate::cl_errors::CustomError;

pub type ClientResult<T> = Result<T, CustomError>;

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

    pub fn mint_nft(
        &self,
        wallet_keypair: &Keypair,
        mint_account_pubkey: &Pubkey,
        token_account_pubkey: &Pubkey,
    ) {
        let wallet_pubkey = wallet_keypair.pubkey();

        let mint_to_instruction: Instruction = spl_token::instruction::mint_to(
            &spl_token::id(),
            &mint_account_pubkey,
            &token_account_pubkey,
            &wallet_pubkey,
            &[&wallet_pubkey],
            1,
        )
        .unwrap();
    
        let (recent_blockhash, _fee_calculator) = client.get_recent_blockhash().unwrap();
        let transaction: Transaction = Transaction::new_signed_with_payer(
            &vec![mint_to_instruction],
            Some(&wallet_pubkey),
            &[wallet_keypair],
            recent_blockhash,
        );
    
        let result = client.send_and_confirm_transaction_with_spinner(&transaction);
        if result.is_ok() {
            println!("Successfully Minted NFT to : {:?}", wallet_pubkey);
    
            upgrade_to_master_edition(
                &wallet_keypair,
                &create_metadata_account(&wallet_keypair, &mint_account_pubkey, &client),
                &mint_account_pubkey,
                &client,
            );
        };
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