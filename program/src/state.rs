//! State transition types
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::pubkey::Pubkey;
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, BorshSchema)]
pub enum MintVersion {
    Uninitialized,
    Initialized,
}

pub const SYMBOL_LEN: usize = 8;
pub const NAME_LEN: usize = 32;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, BorshSchema)]
pub struct Mint {
    pub version: MintVersion,
    pub symbol: [u8; SYMBOL_LEN],
    pub name: [u8; NAME_LEN],
    pub authority: Pubkey,
}

impl Mint {
    pub const LEN: u64 = 73;
    pub fn new(symbol: [u8; 8], name: [u8; NAME_LEN], authority: Pubkey) -> Self {
        Self {
            version: MintVersion::Initialized,
            symbol,
            name,
            authority,
        }
    }
    pub fn is_initialized(&self) -> bool {
        self.version == MintVersion::Initialized
    }
}

pub const URI_LEN: usize = 256;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, BorshSchema)]
pub enum TokenStatus {
    Uninitialized,
    Initialized,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, BorshSchema)]
pub struct Token {
    pub version: TokenStatus,
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub approval: Option<Pubkey>,
}

impl Token {
    pub const LEN: u64 = 98;
}

//NOTE:  BorshSchema can be fixed by wrapping OR with BorshSchema changes with Rust 1.51
//the trait `BorshSchema` is not implemented for `[u8; 256]`
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq)]
pub enum TokenDataStatus {
    Uninitialized,
    Initialized,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct TokenData {
    pub version: TokenDataStatus,
    pub token: Pubkey,
    pub hash: Pubkey,
    pub uri: [u8; URI_LEN],
}

impl TokenData {
    pub const LEN: u64 = 321;

    pub fn get_uri(&self) -> url::Url {
        url::Url::parse(&String::from_utf8(self.uri.to_vec()).unwrap()).unwrap()
    }
}