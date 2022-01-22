use solana_program;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;
pub mod error;
pub mod state;
pub mod instruction;
pub mod processor;

solana_program::declare_id!("CqDwGtvNNgrt5gkk7wqEMqW6T2uahRNVKTaBnGTbNCKz");