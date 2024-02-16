# solcov
Solana Code Coverage CLI Tool

## Installtion & Usage

```bash
# Clone repo
git clone https://github.com/LimeChain/solcov
cd solcov

# Install a nightly version of the compiler (needed to compile with profiling info)
rustup install nightly

# To run the coverage checks on the provided example
cargo run -- --path ./examples/setter

# To install globally as `solcov`
cargo install
solcov --path ./examples/setter
```
