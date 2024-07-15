# solcov
Solana Code Coverage CLI Tool

## Installtion & Usage

```bash
# Install 

# Clone repo
git clone https://github.com/LimeChain/solcov
cd solcov

# Install the llvm-tools rustup component (needed for getting the profiling info on stable)
rustup component add llvm-tools-preview

# To run the coverage checks on the provided example
## With `-C instrument-coverage` (default, works on stable)
cargo run --release -- --path ./examples/setter --coverage-strategy instrument-coverage
## With `-Zprofile` (required nightly) (currently broken, prefer `instrument-coverage`)
cargo run --release -- --path ./examples/setter --coverage-strategy z-profile

# To install globally as `solcov`
cargo install --path .
solcov --path ./examples/setter
```

## Demo screenshot

![image](https://github.com/user-attachments/assets/e2cc4dd9-e288-43f3-8378-a935496c2821)

## Branch coverage

> [!NOTE]
> Branch coverage can be enabled with the `--branch` flag but it requires a recent enough version of the nightly compiler to work
> It is also only supported when using the `instrument-coverage` coverage strategy

<details>
  <summary>There isn't yet a version of the compiler that both supports `branch` coverage and `solana-program` compilation</summary>

  - To support the `rustc` [`coverage-options` setting](https://doc.rust-lang.org/rustc/instrument-coverage.html#-z-coverage-optionsoptions) (telling `rustc` _how to gather coverage information_), we need a recent version of the compiler ([this](https://github.com/rust-lang/rust/pull/122226) (seen in `1.78.0`) for simple branch coverage and [this](https://github.com/rust-lang/rust/pull/123409) (seen in `1.79.0`) for [advanced `mcdc` branch coverage](https://en.wikipedia.org/wiki/Modified_condition/decision_coverage))
  - Our solana programs transitively depend on `ahash`: `solana-program v1.18.1` (latest) -> `borsh v0.9.3` -> `hashbrown v0.11.2` -> `ahash v0.7.7`
      - `solana-program` also [sets](https://github.com/solana-labs/solana/blob/27eff8408b7223bb3c4ab70523f8a8dca3ca6645/sdk/program/Cargo.toml#L12) its `rust-version` to be `1.75.0` for the whole `platform-tools` suite, `solana-program-library` [does too](https://github.com/solana-labs/solana-program-library/blob/8f832e628bac06bf8fa34497ae0b3e0e8c3d0653/rust-toolchain.toml#L2)
  - Unfortunately, since `Rust` removed support for the `stdsimd` feature [here](https://github.com/rust-lang/rust/pull/117372) (seen in `1.78.0`), `ahash v0.7.7` [breaks](https://github.com/tkaitchuck/aHash/issues/200)
  - This is [fixed](https://github.com/tkaitchuck/aHash/pull/183) in `ahash v0.8.0`, but we _cannot_ directly update the version used by `solana-program`.
      - We can try to use `Cargo patches` to force the version of `ahash` but they do not work for transitive dependencies (only for top-level ones, i.e. the ones in our `Cargo.toml`s)
  - The last version of the `Rust` compiler from before the removal of `stdsimd` is `nightly-2024-02-04`, but it does not yet include support for `-Z coverage-options` (introduced roughly a month later)

  Possible long-term solutions:
  - The `solana` ecosystem moves to a newer version of the `Rust` compiler
    Have no details about such intentions, haven't researched, will probably not be soon
  - `Cargo patches` start working for transitive dependencies
    Unlikely, since it would be a nontrivial task to select the exact dependencies you want to patch

  **TLDR**: we either chose to support `branch` coverage or the ability to compile solana programs (IMO the second is a far more important requirement)
</details>

