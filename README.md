# solcov
Solana Code Coverage CLI Tool

## Installtion & Usage

```bash
# Clone repo
git clone https://github.com/LimeChain/solcov
cd solcov

export COMPILER_VERSION="nightly-2023-12-03"

# Install a nightly version of the compiler (needed to compile with profiling info)
rustup install "${COMPILER_VERSION}"

# To run the coverage checks on the provided example
cargo run -- --path ./examples/setter --compiler-version "${COMPILER_VERSION}"

# To install globally as `solcov`
cargo install
solcov --path ./examples/setter --compiler-version "${COMPILER_VERSION}"
```
