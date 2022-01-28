pub mod client;
pub mod cl_errors;

#[cfg(test)]
mod tests {

	use solana_sdk::signer::{
		Signer,
		keypair::{
			read_keypair_file,
			write_keypair_file,
			Keypair
		},
	};
	use solana_sdk::pubkey::Pubkey;
	use std::str::FromStr;
	use crate::client::NftClient;
	
	pub const WALLET_FILE_PATH: &'static str = "wallet.keypair";

	// pub fn get_wallet() -> Keypair {
    // 	let wallet_keypair: Keypair = if let Ok(keypair) = read_keypair_file(WALLET_FILE_PATH) {
    //     	keypair
    // 	} else {
    //     	let new_keypair = Keypair::new();
    //     	write_keypair_file(&new_keypair, WALLET_FILE_PATH).unwrap();
    //     	new_keypair
    // 	};

    // 	return wallet_keypair;
	// }

	// #[test]
	// fn test_program_id() {
	// 	let got = alloy_token_program::id().to_string();

	// 	let want = "D7RRT9SPTuFxXtEJQSP1kSN5me9Q56UsDYAhFsbmXpsa".to_string();

	// 	assert_eq!(want, got, "Alloy Token Program Key: {}, but got: {}", want, got);
	// }

	// #[test]
	// fn test_create_mint_account() {
	// 	let rpc_client: NftClient = NftClient::new().unwrap();
		
	// 	let wallet_keypair = get_wallet();

	// 	let mint_acc = Keypair::new();

	// 	let acc = rpc_client.create_mint_account(&wallet_keypair, &mint_acc, None).unwrap();

	// 	let acc_balance = rpc_client.get_balance(&acc);

	// 	assert_eq!(acc_balance, Ok(1461600));
	// }

	// #[test]
	// fn test_create_token_account() {
	// 	let rpc_client: NftClient = NftClient::new().unwrap();
		
	// 	let wallet_keypair = get_wallet();

	// 	let account_mint_to = Keypair::new();

	// 	let mint_acc = Keypair::new();

	// 	let mint_acc_pubkey = rpc_client.create_mint_account(&wallet_keypair, &mint_acc, None).unwrap();
	// 	let token_acc_pubkey = rpc_client.create_token_account(&wallet_keypair, &account_mint_to, &mint_acc_pubkey).unwrap();

	// 	let token_balance = rpc_client.get_balance(&token_acc_pubkey);

	// 	assert_eq!(token_balance, Ok(2039280));
	// }

	// #[test]
	// fn test_create_alloy_data_accounts() {
	// 	let rpc_client = NftClient::new().unwrap();

	// 	let mint_acc = Keypair::new();
	// 	println!("--> Mint Account: {}", &mint_acc.pubkey());

	// 	let _ = rpc_client.airdrop(&mint_acc.pubkey(), 2_000_000_000);

	// 	let wallet_keypair = get_wallet();
	// 	let _ = rpc_client.airdrop(&wallet_keypair.pubkey(), 2_000_000_000);
	// 	let name = "20MnCr5".to_string();
	// 	let symbol = "ALLOY".to_string();
	// 	let uri = "20MnCr5".to_string();
	// 	let listed_price = 1.5;

	// 	let got = rpc_client.create_alloy_data_accounts(
	// 		name,
	// 		symbol,
	// 		uri,
	// 		listed_price,
	// 		&wallet_keypair,
	// 		&mint_acc.pubkey(),
	// 	);

	// 	let got_latest_id = got.unwrap().0.id as usize;

	// 	let wanted_latest_id = rpc_client.get_total_nfts().unwrap();

	// 	assert_eq!(got_latest_id, wanted_latest_id);
	// }

	// #[test]
	// fn test_get_tokens_by_owner() {
	// 	let rpc_client = NftClient::new().unwrap();

	// 	let got = rpc_client.get_tokens_by_owner(&Pubkey::from_str("4fctzAy7Ay37bz5KfaEfJMVMdRyPXRaU76pWyeiPvTxi").unwrap()).unwrap();

	// 	assert_eq!(2, 5);
	// }

	// #[test]
	// fn test_get_all_alloys() {
	// 	let rpc_client = NftClient::new().unwrap();

	// 	let got = rpc_client.get_all_alloys();

	// 	assert_eq!(2, 5);
	// }

	#[test]
	fn test_get_all_owners_by_alloy_uri() {
		let rpc_client = NftClient::new().unwrap();

		let got = rpc_client.get_all_owners_by_alloy_uri("sae8620".to_string());

		assert_eq!(2, 5);
	}
}