use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    sysvar,
};

use crate::state::{
    NAME_LEN,
    SYMBOL_LEN,
    URI_LEN
};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct MintData {
    pub symbol: [u8; SYMBOL_LEN],
    pub name: [u8; NAME_LEN],
}

impl MintData {
    pub fn new<S: AsRef<str>>(symbol: S, name: S) -> Result<Self, &'static str> {
        let symbol = symbol.as_ref().as_bytes();
        let name = name.as_ref().as_bytes();
        if symbol.len() > SYMBOL_LEN || name.len() > NAME_LEN {
            return Err("symbol or name too long");
        }
        let mut this = Self {
            name: [0; NAME_LEN],
            symbol: [0; SYMBOL_LEN],
        };
        // any shorter notation
        let (left, _) = this.name.split_at_mut(name.len());
        left.copy_from_slice(name);

        let (left, _) = this.symbol.split_at_mut(symbol.len());
        left.copy_from_slice(symbol);

        Ok(this)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct TokenDataArgs {
    pub hash: Pubkey,
    pub uri: [u8; URI_LEN],
}

impl TokenDataArgs {
    pub fn new(hash: Pubkey, uri: url::Url) -> Result<Self, &'static str> {
        let uri = uri.as_str().as_bytes();
        if uri.len() > URI_LEN {
            return Err("uri too long");
        }
        let mut this = Self {
            hash,
            uri: [0; URI_LEN],
        };
        let (left, _) = this.uri.split_at_mut(uri.len());
        left.copy_from_slice(uri);
        Ok(this)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum NftInstruction {
    InitializeMint(MintData),
    InitializeToken(TokenDataArgs),
    Transfer,
    Approve,
    Burn,
}

impl NftInstruction {
    pub fn initialize_mint(
        mint_account: &Pubkey,
        data: MintData,
        authority: &Pubkey,
    ) -> Instruction {
        let data = NftInstruction::InitializeMint(data);
        let accounts = vec![
            AccountMeta::new(*mint_account, false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(*authority, true),
        ];
        Instruction::new_with_borsh(crate::id(), &data, accounts)
    }

    pub fn initialize_token(
        token: &Pubkey,
        token_data: &Pubkey,
        mint: &Pubkey,
        owner: &Pubkey,
        input: TokenDataArgs,
        mint_authority: &Pubkey,
    ) -> Instruction {
        let data = NftInstruction::InitializeToken(input);
        let accounts = vec![
            AccountMeta::new(*token, false),
            AccountMeta::new(*token_data, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(*owner, false),
            AccountMeta::new_readonly(*mint_authority, true),
        ];
        Instruction::new_with_borsh(crate::id(), &data, accounts)
    }

    pub fn transfer(token: Pubkey, new_owner: Pubkey, approval_or_owner: Pubkey) -> Instruction {
        let data = NftInstruction::Transfer;
        let accounts = vec![
            AccountMeta::new(token, false),
            AccountMeta::new_readonly(new_owner, false),
            AccountMeta::new_readonly(approval_or_owner, true),
        ];
        Instruction::new_with_borsh(crate::id(), &data, accounts)
    }

    pub fn approve(token: Pubkey, new_approval: Pubkey, approval_or_owner: Pubkey) -> Instruction {
        let data = Self::Approve;
        let accounts = vec![
            AccountMeta::new(token, false),
            AccountMeta::new_readonly(new_approval, false),
            AccountMeta::new_readonly(approval_or_owner, true),
        ];
        Instruction::new_with_borsh(crate::id(), &data, accounts)
    }

    pub fn burn(token: Pubkey, token_data: Pubkey, approval_or_owner: Pubkey) -> Instruction {
        let data = Self::Burn;
        let accounts = vec![
            AccountMeta::new(token, false),
            AccountMeta::new(token_data, false),
            AccountMeta::new(approval_or_owner, true),
        ];
        Instruction::new_with_borsh(crate::id(), &data, accounts)
    }
}
