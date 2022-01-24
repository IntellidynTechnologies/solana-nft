pub mod client;
pub mod cl_errors;

#[cfg(test)]

mod tests {

	use solana_sdk::signer::{Signer, keypair::{read_keypair_file, write_keypair_file, Keypair}};
	use solana_program::pubkey::Pubkey;
	use alloy_token_program::state::AlloyData;
	use crate::client::NftClient;
	use std::str::FromStr;
	
	pub const WALLET_FILE_PATH: &'static str = "wallet.keypair";

	pub fn get_wallet() -> Keypair {
    	let wallet_keypair: Keypair = if let Ok(keypair) = read_keypair_file(WALLET_FILE_PATH) {
        	keypair
    	} else {
        	let new_keypair = Keypair::new();
        	write_keypair_file(&new_keypair, WALLET_FILE_PATH).unwrap();
        	new_keypair
    	};

    	return wallet_keypair;
	}

	#[test]
	fn test_program_id() {
		let got = alloy_token_program::id().to_string();

		let want = "GKAL7MXHwHB7JqLf6EQWpAE29zdfkPqdckpsYY77JyV1".to_string();

		assert_eq!(want, got, "Alloy Token Program Key: {}, but got: {}", want, got);
	}

	#[test]
	fn test_create_alloy_data_accounts() {
		let rpc_client = NftClient::new().unwrap();

		let program_key = alloy_token_program::id();
		let add_balance_to_program = rpc_client.airdrop(&program_key, 2);
		println!("Signature: {}", &add_balance_to_program.unwrap());

		let owner = Keypair::new();
		let name = "SAE8620".to_string();
		let uri = "sae8620".to_string();
		let last_price = 20;
		let listed_price = 24;
		let wallet_keypair = get_wallet();
		
		let add_balance_to_owner = rpc_client.airdrop(&owner.pubkey(), 2);
		println!("Signature: {}", &add_balance_to_owner.unwrap());

		let add_balance_to_wallet = rpc_client.airdrop(&wallet_keypair.pubkey(), 2);
		println!("Signature: {}", &add_balance_to_wallet.unwrap());

		let got = rpc_client.create_alloy_data_accounts(
			&wallet_keypair,
			name,
			uri,
			last_price,
			listed_price,
			&owner,
			Some(1),
		);

		let want: (AlloyData, Pubkey) = (
			AlloyData {
				id: 1,
				name: "SAE8620".to_string(),
				uri:"sae8620".to_string(),
				last_price: 20,
				listed_price: 24,
				owner_address: Pubkey::from_str("3JS1m9mNKi28rPb9B84xRLN5BYkhbfzJLQP6iS9Q2RWx").unwrap()
			},
			Pubkey::from_str("BjMraAFVTYT5n1XibhabgDVTGU9qmuopwjTMNRQ6FYE6").unwrap()
		);

		assert_eq!(*&got.as_ref(), Ok(&want), "\nGot -> {:?}, \nWant -> {:?}", got, want);

		let got = rpc_client.get_all_alloys();

		let want = 0;

		assert_eq!(got.len(), want);
	}
}