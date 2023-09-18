# sim-validator-assignment

Simulate validator assignment to get an estimated probability of shard corruption.

# Algorithm for validator assignment

TODO

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
cargo run -- --help
# Run a simulation with your parameters.
cargo run -- --num-blocks <...>

# Alternatively, run a simulation defined in the `Makefile`.
make <target>
```

# Development

Notes regarding the development workflow can be found in [`Development.md`](./Development.md).

# TODO

- setup cargo clippy and rust-fmt
- setup CI
