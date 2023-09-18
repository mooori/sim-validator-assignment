# Workflow

Tests can be run with `cargo test`. When adding a new a new snapshot assertion (snapshot testing is described below), the following command is recommended:

```
cargo insta test --review
```

which allows reviewing the newly created snapshot file in the terminal. This requires `cargo insta` which can be installed with:

```
cargo install cargo-insta
```

# Testing

Many tests are based on snapshots using the [`insta`](https://insta.rs) crate, which facilitates comparing a value to a reference value. When executing a test that asserts snapshots for the first time, the serializations of the values to be asserted are stored in files. The developer reviews these files and accepts the snapshots if they corresponds to the expected values. If the actual value deviates from the expected value in a future execution of the test, the developer reviews the diff. Changes in expected value can be due to refactors or bugs, for example.

## Motivation for using `insta`

It allows fast iteration on tests. For instance, to test a function that returns `Vec<Validator>>`, it is sufficient to invoke `insta::assert_yaml_snapshot!(validators)` and review the serialized `Vec<Validator>`. Usually this is more developer friendly than mocking or hand-writing the expected value of `validators`.

## Snapshot files

Snapshot files are located in the `snapshot` subdirectory of the directory that contains the source file. The naming of the file follows the schema `<crate>__<module_1>__<module_n>__<test_function>.snap` where modules can be nested (`module_i`).
