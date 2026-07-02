# aperture-grpc-proto

Generated Rust protobuf and gRPC bindings for Aperture's lightweight decoded
transaction stream.

The service path is:

```text
/aperture.Aperture/SubscribeTransactions
/aperture.Aperture/SubscribeTransactionBatches
```

The stream is pre-execution. It carries decoded transaction message data from
Aperture's shred/deshred pipeline and does not include execution status, logs,
balances, rewards, inner instructions, or compute units.

## Install

```toml
[dependencies]
aperture-grpc-proto = "0.2.0"
```

For unreleased development builds:

```toml
[dependencies]
aperture-grpc-proto = { git = "https://github.com/dysnix/aperture-grpc-proto" }
```

## Usage

```rust,ignore
use aperture_grpc_proto::{
    SubscribeTransactionsRequest, VoteFilter,
    aperture_client::ApertureClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ApertureClient::connect("http://127.0.0.1:10102").await?;
    let request = SubscribeTransactionsRequest {
        vote: VoteFilter::NonVoteOnly as i32,
        ..Default::default()
    };
    let mut stream = client.subscribe_transactions(request).await?.into_inner();

    while let Some(tx) = stream.message().await? {
        println!("slot={} index={} signatures={}", tx.slot, tx.index, tx.signatures.len());
    }

    Ok(())
}
```

Use `SubscribeTransactionBatches` to receive available transactions grouped in
one gRPC message:

```rust,no_run
# use aperture_grpc_proto::{SubscribeTransactionsRequest, aperture_client::ApertureClient};
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let mut client = ApertureClient::connect("http://127.0.0.1:10102").await?;
let request = SubscribeTransactionsRequest {
    signatures_only: true,
    ..Default::default()
};
let mut stream = client.subscribe_transaction_batches(request).await?.into_inner();
while let Some(batch) = stream.message().await? {
    println!("batch transactions={}", batch.transactions.len());
}
# Ok(())
# }
```

Clients resolve instruction account indexes by concatenating:

```text
static_account_keys + loaded_writable_addresses + loaded_readonly_addresses
```

Filters use raw bytes:

- `signature`: optional 64-byte primary signature filter.
- `account_include`: 32-byte pubkeys, match any known account.
- `account_exclude`: 32-byte pubkeys, reject if any known account matches.
- `account_required`: 32-byte pubkeys, require all known accounts.
- `signatures_only`: omit message account/instruction payloads and keep only
  slot/index/vote/timestamp/version/signatures.
