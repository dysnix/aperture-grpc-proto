# aperture-grpc-proto

Generated Rust protobuf and gRPC bindings for Aperture's lightweight decoded
transaction stream.

The service path is:

```text
/aperture.Aperture/SubscribeTransactions
/aperture.Aperture/SubscribeTransactionBatches
```

By default the stream is pre-execution. Clients can set
`SubscribeTransactionsRequest.include_simulation` to append Agave Bank
simulation status, compute units, bank slot, and timing to each emitted
transaction. Additional details require `simulation_config`.

Each full transaction also reports `alt_resolution` as `"FULL"` when its
account list is complete or `"PARTIAL"` when one or more address lookup table
entries could not be resolved. The field is absent when ALT resolution is not
available.

## Transaction versions

Version 0.5 adds v1 transactions and `TransactionConfig`. V1 is delivered
by default on both RPCs with the existing subscription request: no opt-in or
request-schema change is needed. Upgrade generated clients to understand the
new enum value and config; older protobuf decoders ignore the additional field.

`TransactionVersion` values remain `Legacy = 0`, `V0 = 1`; `V1 = 2` is new.
Full v1 responses carry `transaction_config`, even if all its fields are absent.
Priority fee is **total lamports**. Absent CU and loaded account data limits
mean zero; absent heap size means 32 KiB. Legacy/v0 have no config. In v1 all
accounts are static; no ALT addresses are loaded. Signatures-only responses
retain the version but omit config along with the message payload.

## Install

```toml
[dependencies]
aperture-grpc-proto = "0.6.1"
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
        include_simulation: true,
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

Check `alt_resolution` before treating that concatenated list as complete.
Transactions that do not use lookup tables report `"FULL"`; an absent value
means the server could not provide resolution status.

Filters use raw bytes:

- `signature`: optional 64-byte primary signature filter.
- `account_include`: 32-byte pubkeys, match any known account.
- `account_exclude`: 32-byte pubkeys, reject if any known account matches.
- `account_required`: 32-byte pubkeys, require all known accounts.
- `signatures_only`: omit message account/instruction payloads and keep only
  slot/index/vote/timestamp/version/signatures.
- `include_simulation`: wait for simulation and append a
  `TransactionSimulation` to each transaction. This can be combined with
  `signatures_only` for a lightweight signature-and-result stream.

## Simulation details

`include_simulation: true` returns status, error, compute units, bank slot and
simulation timing. Logs and other details are disabled by default.

Set `simulation_config` to choose details on either RPC. Its presence enables
simulation; all `include` flags default to false. An explicit config takes
precedence over `include_simulation`.

```json
{
  "simulation_config": {
    "include": {
      "compute_units": true,
      "account_deltas": false,
      "token_balance_deltas": true,
      "inner_instructions": false,
      "logs": true,
      "return_data": false
    }
  }
}
```

| Include field | Response |
| --- | --- |
| `compute_units` | Consumed compute units |
| `account_deltas` | Writable account pre/post lamports, owner, executable and rent epoch |
| `token_balance_deltas` | Token pre/post mint, authority, program ID and raw amount |
| `inner_instructions` | CPI instructions with resolved program/account keys and stack height |
| `logs` | Simulation logs |
| `return_data` | Program return data |

Deltas contain only changed writable accounts. Account entries carry
`changed: true`, including changes to account data; raw data is not returned.
Missing pre/post state represents account creation, closure or token conversion.
Token amounts have no decimals or UI conversion and require no mint lookup.
Token-2022 reports the base token balance, without extension balances.
Failed simulations return rollback state with applicable fee/nonce changes.

Optional `simulation_config.account_include` and `owner_include` filters select
deltas. Both accept 32-byte pubkeys (base64 in protobuf JSON); empty lists allow
all accounts. When both are set, both must match. Owner filters match either
pre/post account owner program. Logs and CPI are unaffected by these filters.
Simulation details also work with `signatures_only`.
