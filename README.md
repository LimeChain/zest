# Zest

Solana Code Coverage CLI Tool

## Demo screenshot

![image](https://github.com/user-attachments/assets/e2cc4dd9-e288-43f3-8378-a935496c2821)

## Installtion

```bash
# Clone repo
git clone https://github.com/LimeChain/zest
cd zest

# Install globally as `zest`
cargo install --path .
```

## Usage

```bash
# Move into the target project
cd ./examples/setter/anchor
# This will run coverage for the example using the `instrument-coverage` strategy without `branch` info
zest

# Path to the target project can also be specified using the `--path` option
zest cov --path ./examples/setter/anchor

# Configuration options can also be read from a (TOML) config file (`zest-coverage.toml` by default)
cat <<TOML > my_zest_config.toml
path = "./examples/setter/anchor"
branch = true
# tests = ["integration"]
# output_types = ["lcov", "html"]
TOML

# Which would run with
#  `coverage_strategy` being `instrument-coverage`       (Default)
#               `path` being `./examples/setter/anchor/` (from config file)
#             `branch` being `false`                     (CLI override)
zest cov --config ./my_zest_config.toml --branch false
```

> [!NOTE]
> Check `zest --help` and `zest coverage --help` for more info

> [!NOTE]
> More info on the different strategies can be found [here](https://doc.rust-lang.org/beta/rustc/instrument-coverage.html)

## Program compatibility

Currently, `zest` only supports testing programs, written in Rust, with tests written in Rust (usually using [solana-program-test](https://crates.io/crates/solana-program-test), as opposed to the *classic* `Typescript` tests), which do not depend on the `cargo-{build,test}-sbf` toolchain. A.K.A if `cargo test` works for you (not `cargo test-sbf`), then `zest` will too

Here's a small list of publicly available Solana programs that we've tested if they work with `zest` or not:

Works on:

- [raydium-clmm](https://github.com/raydium-io/raydium-clmm)
- [serum-dex](https://github.com/jup-ag/serum-dex)
- [token-vesting](https://github.com/staratlasmeta/token-vesting)

### Compatibility requirements

How to make sure `zest` works for your program:

1. Make sure you're using a Rust framework ([solana-program-test](https://crates.io/crates/solana-program-test) or similar, like [liteSVM](https://github.com/LiteSVM/litesvm)) for your testing purposes
2. Make sure your tests are runnable by just `cargo test`
    
    This is done by supplying your program's `processor` (the `process_instruction` function) directly when adding it to the test validator 
    
    ```rust
    let mut validator = ProgramTest::default();
    validator.add_program(
        "counter_solana_native",
        counter_solana_native::ID,
        processor!(counter_solana_native::process_instruction),
    );
    ```
    
    That requirement is incompatible with `shank` framework, since it puts a type constraint on the `processor` function. 

> [!NOTE]
> That happens because of the `context` function from [`ShankContext`](https://docs.rs/shank/0.4.2/shank/derive.ShankContext.html), seen in their [example](https://docs.rs/shank/0.4.2/shank/derive.ShankContext.html#example) (the `'a` lifetime), which breaks the compatibility (and thus makes it testable only in *`sbf` mode*).

## Branch coverage

> [!NOTE]
> Branch coverage can be enabled with the `--branch` flag but it requires a recent enough version of the nightly compiler to work.
> It is also only supported when using the `instrument-coverage` coverage strategy (default).

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
