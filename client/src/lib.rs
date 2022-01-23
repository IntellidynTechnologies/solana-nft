pub mod client;
pub mod cl_errors;

#[cfg(test)]

mod tests {

	use solana_sdk::signer::{Signer, keypair::{read_keypair_file, write_keypair_file, Keypair}};
	use solana_program::pubkey::Pubkey;
	use alloy_token_program::state::AlloyData;
	use crate::client::NftClient;
	
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

		let want = "9Fnz4RCe3SC3jNyGvkaHTZ4P51Cd5DVfnkHKaRcmmoRw".to_string();

		assert_eq!(want, got, "Alloy Token Program Key: {}, but got: {}", want, got);
	}

	#[test]
	fn test_create_alloy_data_accounts() {
		let rpc_client = NftClient::new().unwrap();

		let payer = Keypair::new();
		let name = "SAE8620".to_string();
		let uri = "sae8620".to_string();
		let last_price = 20;
		let listed_price = 24;
		let wallet_keypair = get_wallet();

		let got = rpc_client.create_alloy_data_accounts(
			&payer,
			name,
			uri,
			last_price,
			listed_price,
			&wallet_keypair
		);

		let want: (AlloyData, Pubkey) = (
			AlloyData {
				id: 1,
				name: "SAE8620".to_string(),
				uri:"sae8620".to_string(),
				last_price: 20,
				listed_price: 24,
				owner_address: wallet_keypair.pubkey()
			},
			Pubkey::new_unique()
		);

		assert_eq!(got, want);
	}

	// #[test]
	// fn test_solana_nft() {
	// 	let rpc_client = NftClient::new().unwrap();

	// 	let wallet_keypair = get_wallet();
	// 	let wallet_pubkey = wallet_keypair.pubkey();

	// 	let program_key = alloy_token_program::id();
 //    	println!("{:?}", program_key);

 //    	let before_wallet_balance = rpc_client.get_balance(&wallet_keypair);

 //    	let alloy_data_account = rpc_client.create_alloy_data_accounts(
 //    		&wallet_keypair,
 //    		"SAE8620".to_string(),
 //    		"sae8620".to_string(),
 //    		20,
 //    		24,
 //    		&wallet_keypair,
 //    	);
 //    	let after_wallet_balance = rpc_client.get_balance(&wallet_keypair);

 //    	assert!(before_wallet_balance > after_wallet_balance);
	// }
}