#![doc = include_str!("../README.md")]

/// Aperture lightweight decoded transaction stream protobuf package.
pub mod aperture {
    tonic::include_proto!("aperture");
}

pub use aperture::{
    CompiledInstruction, DecodedTransaction, DecodedTransactionBatch, MessageHeader,
    SubscribeTransactionsRequest, TransactionVersion, VoteFilter, aperture_client, aperture_server,
};
