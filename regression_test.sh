#!/bin/bash
#// regression test before release

# all examples
cargo run --release --example 2>&1 | grep -E '^ ' | grep -v basic_consumer | xargs -n1 cargo run --release --all-features --example

# Test all features combinations
cargo test 
cargo test -F traces
cargo test -F compliance_assert
cargo test -F tls
cargo test -F urispec



# clippy, warnings not allowed
cargo clippy --all-features -- -Dwarnings

# docs build
cargo doc -p amqprs --all-features --open

# cargo msrv
cargo msrv

# dry-run publish
cargo publish -p amqprs --all-features --dry-run