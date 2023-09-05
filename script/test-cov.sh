CARGO_INCREMENTAL=0 LLVM_PROFILE_FILE="$PWD/target/profile/cargo-test-%p-%m.profraw" RUSTFLAGS="-C instrument-coverage" cargo test --features css-block-lint

grcov target/profile/ --binary-path target/debug/deps -s unstyled_macro/src -o target/coverage -t html --branch --ignore-not-existing --ignore '*/src/lib.rs'

