# solcov
Limechain Solana Code Coverage CLI

This CLI is roughly based on [this script](https://github.com/solana-labs/solana-program-library/blob/4eb50ed57111dde77c1460778966b7fe559f1513/coverage.sh) but with `grcov` brought in as a `rust` dependency, instead of depending on it existing as an external `cli` tool.

# Running

You can directly run the binary using

```bash
cargo run -- --path path/to/solana/program/with/rust/tests
```

Or `cargo install` it first and then run it as `solcov path/to/solana/program/with/rust/tests`
