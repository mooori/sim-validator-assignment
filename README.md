# sim-validator-assignment

Simulate validator assignment to get an estimated probability of shard corruption.

# Usage

Assuming [Rust](https://www.rust-lang.org/tools/install) is installed on the system, this crate can be downloaded and built with:

```
git clone https://github.com/mooori/sim-validator-assignment.git
cd sim-validator-assignment
cargo build
```

Hints on how to run a simulation:

```
# Show a description of the simulation parameters.
cargo run -p sim-validator-assignment -- --help
# Run a simulation with your parameters.
cargo run -p sim-validator-assignment -- run --num-blocks <...>

# Alternatively, run a simulation defined in the `Makefile`.
make <target>
```

More detailed explanation for particular blockchains follow below.

## NEAR

To print the docs for any of the commands used below:

```bash
cargo run -p sim-validator-assignment -- <command> --help
```

### 1: Download validator data

To download the latest validator data and store it in `validator_data.json`:

```bash
cargo run -p sim-validator-assignment -- \
	download \
	--protocol near	\
	--rpc-url 'https://rpc.mainnet.near.org' \
	--out ./validator_data.json
```

- It is possible to download validator data for a specific epoch by providing the `--block-height` parameter. Note that for NEAR it must be the last block of an epoch, otherwise the RPC returns an (rather opaque) error.
- To download validator data for another network (e.g.) testnet, use a corresponding `--rpc-url`.

### 2: Make some validators malicious

The purpose of the simulation is determining the estimated probability of shard corruption, so some validators should be malicious. To make a validator malicious, set its `is_malicious` value to true in `validator_data.json`.

Some basic statistics for the validator population can be printed with:

```bash
cargo run -p sim-validator-assignment -- \
	seat-stats \
	--validator-data ./validator_data.json \
	--stake-per-seat 1140000000000000000000000000000 \
	--include-partial-seats
```

### 3: Run the simulation

```bash
cargo run -p sim-validator-assignment -- \
	run \
	--num-blocks 10000000000 \
	--num-shards 6 \
	--seats-per-shard 68 \
	--stake-per-seat 1140000000000000000000000000000 \
	--max-malicious-stake-per-shard 2/3 \
	--include-partial-seats \
	--validator-data ./validator_data.json
```

Depending on the parameters the simulation may run for a long time. Periodically the number of `corrupted_shards/simulated_shards` is printed to the console.

# Algorithm for validator assignment

Validator assignment is based on a random shuffle of validator seats. The number of seats a validator gets assigned is a function of its stake and simulation parameters. All validators' seats are collected in a vector which is then shuffled and shuffled seats are assigned to shards.

## Randomness

For now, this crate uses [`fastrand`] to shuffle the vector of seats. For the future, it is possible to allow users to chose between different crates that provide randomness. For instance with feature flags at compile time or with CLI parameters when initiating a simulation.

# Other commands

Besides running simulations, this tool offers further commands allowing to download and analyze validator data:

```
Commands:
  run         Runs a simulation
  download    Downloads valdiator data
  seat-stats  Prints seat stats
  help        Print this message or the help of the given subcommand(s)
```

# Development

Notes regarding the development workflow can be found in [`Development.md`](./Development.md).
