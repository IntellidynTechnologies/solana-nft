pub mod client;
pub mod cl_errors;

#[cfg(test)]

mod tests {

	use solana_sdk::{
		signer::{Signer, keypair::{read_keypair_file, write_keypair_file, Keypair}},
		pubkey::Pubkey
	};
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

		let want = "D7RRT9SPTuFxXtEJQSP1kSN5me9Q56UsDYAhFsbmXpsa".to_string();

		assert_eq!(want, got, "Alloy Token Program Key: {}, but got: {}", want, got);
	}

	#[test]
	fn test_create_mint_account() {
		let rpc_client: NftClient = NftClient::new().unwrap();
		
		let wallet_keypair = get_wallet();
		
		let add_balance_to_owner = rpc_client.airdrop(&owner.pubkey(), 2);
		println!("Signature: {}", &add_balance_to_owner.unwrap());

		let mint_acc = Keypair::new();

		let acc = rpc_client.create_mint_account(&wallet_keypair, &mint_acc, None).unwrap();

		let acc_balance = rpc_client.get_balance(&acc);

		assert_eq!(acc_balance, Ok(1461600));
	}

	#[test]
	fn test_create_token_account() {
		let rpc_client: NftClient = NftClient::new().unwrap();
		
		let wallet_keypair = get_wallet();

		let account_mint_to = Keypair::new();

		let mint_acc = Keypair::new();

		let mint_acc_pubkey = rpc_client.create_mint_account(&wallet_keypair, &mint_acc, None).unwrap();
		let token_acc_pubkey = rpc_client.create_token_account(&wallet_keypair, &account_mint_to, &mint_acc_pubkey).unwrap();

		let token_balance = rpc_client.get_balance(&token_acc_pubkey);

		assert_eq!(token_balance, Ok(2039280));
	}

	// #[test]
	// fn test_create_alloy_data_accounts() {
	// 	let rpc_client = NftClient::new().unwrap();

	// 	let payer = Keypair::from_base58_string("1111111111111111111111111111111111111111111111111111111111111111");
	// 	let name = "SAE8620".to_string();
	// 	let uri = "sae8620".to_string();
	// 	let last_price = 20;
	// 	let listed_price = 24;
	// 	let wallet_keypair = get_wallet();

	// 	rpc_client.airdrop(&payer.pubkey(), 2);
	// 	rpc_client.airdrop(&wallet_keypair.pubkey(), 2);

	// 	let got = rpc_client.create_alloy_data_accounts(
	// 		&wallet_keypair,
	// 		name,
	// 		uri,
	// 		last_price,
	// 		listed_price,
	// 		&payer,
	// 		Some(1),
	// 	);

	// 	let want: (AlloyData, Pubkey) = (
	// 		AlloyData {
	// 			id: 1,
	// 			name: "SAE8620".to_string(),
	// 			uri:"sae8620".to_string(),
	// 			last_price: 20,
	// 			listed_price: 24,
	// 			owner_address: payer.pubkey()
	// 		},
	// 		Pubkey::from_str("AsVr8Cd6HwtRtZ3ypX4yjcw96aMVc5si8LRN5P3ydYBJ").unwrap()
	// 	);

	// 	assert_eq!(*&got.as_ref(), Ok(&want), "\nGot -> {:?}, \nWant -> {:?}", got, want);

	// 	let got = rpc_client.get_all_alloys();

	// 	let want = 0;

	// 	assert_eq!(got.len(), want);
	// }
}