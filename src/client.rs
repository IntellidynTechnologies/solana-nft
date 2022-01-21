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
    pub fn new(rpc_url: String) -> Result<Self, CustomError> {
        Ok(Client {
            client: RpcClient::new_with_commitment(
                rpc_url,
                CommitmentConfig::confirmed()
            )
        })
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
    }
}