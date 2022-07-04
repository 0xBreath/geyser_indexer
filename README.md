# Solana Snapshot ETL 📸

<sub>Built with 🦀 at <em>REDACTED</em></sub>

**`solana-snapshot-etl` efficiently extracts all accounts in a snapshot** to load them into an external system.

## Motivation

Solana nodes periodically backup their account database into a `.tar.zst` "snapshot" stream.
If you run a node yourself, you've probably seen a snapshot file such as this one already:

```
snapshot-139240745-D17vR2iksG5RoLMfTX7i5NwSsr4VpbybuX1eqzesQfu2.tar.zst
```

A full snapshot file contains a copy of all accounts at a specific slot state (in this case slot `139240745`).

Historical accounts data is relevant to blockchain analytics use-cases and event tracing.
Despite archives being readily available, the ecosystem was missing an easy-to-use tool to access snapshot data.

## Usage

### As a command-line tool

The standalone command-line tool can export data to a Geyser plugin.

Create Postgres role "solana" with postgres CLI.

```shell
psql postgres
CREATE ROLE solana LOGIN SUPERUSER;
\q
```

Create Postgres database.

```shell
createdb geyser
chmod u+x ./scripts/reset_database.sh
./scripts/reset_database.sh
```

Build from source.

```shell
cargo build --release --bin solana-snapshot-etl --features standalone
```

Unpack a snapshot.

```shell
mkdir ./unpacked_snapshot
cd ./unpacked_snapshot
tar -xf ../deps/snapshot.tar.bz2
cd ../
```

**Replicate accounts to a Geyser plugin.**

```shell
./target/release/solana-snapshot-etl ./unpacked_snapshot --geyser plugin-config.json
```

### As a library

**TODO: Functions**
Function to trigger slot data dump to database.
Function to query database by account params.
Function to load account data into model (Star Atlas ship program for example).

**TODO: Implementation**
[https://github.com/holaplex/indexer/blob/dev/crates/core/src/db/models.rs]
Define account.data as a model, create a Postgres table.

[https://github.com/holaplex/indexer/blob/dev/crates/indexer/src/geyser/accounts/candy_machine.rs]
Processor to deserialize account.data according to model, save to Postgres table.

[https://github.com/holaplex/indexer/blob/dev/crates/core/src/db/schema.rs]
Define query interface with Diesel, a Rust abstraction over SQL.

**TODO: Docker**
Postgres database.
Cargo build. 
Read snapshot from validator (validator opens websocket to read snapshot from tar file?).
Start local validator or connect to devnet/mainnet validator.

