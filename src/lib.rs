#![doc = include_str!("../README.md")]

/// Aperture lightweight decoded transaction stream protobuf package.
pub mod aperture {
    tonic::include_proto!("aperture");
}

pub use aperture::{
    CompiledInstruction, DecodedTransaction, DecodedTransactionBatch, MessageHeader,
    SimulationStatus, SubscribeTransactionsRequest, TransactionReturnData, TransactionSimulation,
    TransactionVersion, VoteFilter, aperture_client, aperture_server,
};

#[cfg(test)]
mod tests {
    use {super::DecodedTransaction, prost::Message};

    #[derive(Clone, PartialEq, Message)]
    struct OldDecodedTransaction {
        #[prost(uint64, tag = "1")]
        slot: u64,
    }

    #[test]
    fn old_client_ignores_alt_resolution() {
        let current = DecodedTransaction {
            slot: 42,
            alt_resolution: Some("PARTIAL".to_string()),
            ..DecodedTransaction::default()
        };

        let old = OldDecodedTransaction::decode(current.encode_to_vec().as_slice())
            .expect("old schema should ignore the new field");

        assert_eq!(old.slot, 42);
    }

    #[test]
    fn old_server_payload_has_absent_alt_resolution() {
        let old = OldDecodedTransaction { slot: 42 };

        let current = DecodedTransaction::decode(old.encode_to_vec().as_slice())
            .expect("current schema should decode the old payload");

        assert_eq!(current.slot, 42);
        assert_eq!(current.alt_resolution, None);
    }
}
