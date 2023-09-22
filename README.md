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
cargo run -p sim-validator-assignment -- --num-blocks <...>

# Alternatively, run a simulation defined in the `Makefile`.
make <target>
```

# Algorithm for validator assignment

Validator assignment is based on a random shuffle of validator seats. The number of seats a validator gets assigned is a function of its stake and simulation parameters. All validators' seats are collected in a vector which is then shuffled and shuffled seats are assigned to shards.

## Randomness

For now, this crate uses [`fastrand`] to shuffle the vector of seats. For the future, it is possible to allow users to chose between different crates that provide randomness. For instance with feature flags at compile time or with CLI parameters when initiating a simulation.

# Development

Notes regarding the development workflow can be found in [`Development.md`](./Development.md).
