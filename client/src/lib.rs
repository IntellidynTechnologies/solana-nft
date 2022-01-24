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

		let want = "9Fnz4RCe3SC3jNyGvkaHTZ4P51Cd5DVfnkHKaRcmmoRw".to_string();

		assert_eq!(want, got, "Alloy Token Program Key: {}, but got: {}", want, got);
	}

	#[test]
	fn test_create_alloy_data_accounts() {
		let rpc_client = NftClient::new().unwrap();

		let payer = Keypair::from_base58_string("1111111111111111111111111111111111111111111111111111111111111111");
		let name = "SAE8620".to_string();
		let uri = "sae8620".to_string();
		let last_price = 20;
		let listed_price = 24;
		let wallet_keypair = get_wallet();

		rpc_client.airdrop(&payer.pubkey(), 2);
		rpc_client.airdrop(&wallet_keypair.pubkey(), 2);

		let got = rpc_client.create_alloy_data_accounts(
			&wallet_keypair,
			name,
			uri,
			last_price,
			listed_price,
			&payer,
			Some(1),
		);

		let want: (AlloyData, Pubkey) = (
			AlloyData {
				id: 1,
				name: "SAE8620".to_string(),
				uri:"sae8620".to_string(),
				last_price: 20,
				listed_price: 24,
				owner_address: payer.pubkey()
			},
			Pubkey::from_str("AsVr8Cd6HwtRtZ3ypX4yjcw96aMVc5si8LRN5P3ydYBJ").unwrap()
		);

		assert_eq!(*&got.as_ref(), Ok(&want), "\nGot -> {:?}, \nWant -> {:?}", got, want);

		let got = rpc_client.get_all_alloys();

		let want = 0;

		assert_eq!(got.len(), want);
	}
}