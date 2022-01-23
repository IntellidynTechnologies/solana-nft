use solana_client::{
    client_error::ClientError,
    rpc_client::RpcClient
};

use solana_sdk::{
    commitment_config::CommitmentConfig,
    message::Message,
};
use solana_program::{
    pubkey::Pubkey,
    instruction::Instruction
};

use crate::error::CustomError;

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

    pub fn create_mint_account(
        &self,
        program_id: &Pubkey,
        data_len: usize,
        wallet_pk: &Pubkey
    ) -> Result<Pubkey, CustomError> {

        let mint_public_key = Pubkey::new_unique();
        println!(":{?}", mint_public_key);
        
        //minimum balance for rent exemption
        let lamports_requirment = self.client.get_minimum_balance_for_rent_exemption(data_len)?;

        //create account instruction

        //initialize mint instruction

        //latest blockhash
        let latest_blockhash = self.client.get_latest_blockhash().unwrap();

        //transaction

        //result
        let result = self.client.send_and_confirm_transaction_with_spinner(&transaction);

        //check result

        //return mint account pubkey
        Ok(mint_public_key)
    }

    pub fn get_balance_for_rent_exemption(&self, data_len: usize, instruction: &[Instruction], payer: &Pubkey) -> Result<u64, CustomError> {
        let acc_fee = self.client.get_minimum_balance_for_rent_exemption(data_len);
        let hash = self.client.get_latest_blockhash()?;

        let msg = Message::new_with_blockhash(instruction, Some(payer), &hash);

        let fee_for_msg = self.client.get_fee_for_message(&msg)?;

        let txn_fee = fee_for_msg * 100;

        Ok(txn_fee + acc_fee.unwrap())
    }

    pub fn get_owner_balance(&self, owner: &Pubkey) -> Result<u64, ClientError> {
        self.client.get_balance(owner)
    }

    pub fn airdrop(&self, owner: &Pubkey, lamports: u64) -> Result<(), CustomError> {
        let sig = self.client.request_airdrop(owner, lamports)?;

        loop {
            let confirmed = self.client.confirm_transaction(&sig)?;

            if confirmed {
                break;
            }
        }
        Ok(())
    }

    pub fn create_nft_account(&self, data_len: usize, program_id: &Pubkey, user_id: &Pubkey,) -> Result<(), CustomError> {
        let account_pubkey = Pubkey::create_with_seed(user_id, "NFT", program_id);

        let lamport_req = self.client.get_minimum_balance_for_rent_exemption(data_len)?;

        Ok(())
    }
}