check:
    @- echo "Linting files..."
    - cargo fmt -- --check
    - cargo clippy -- -D warnings

test:
    - cargo test

run-jlox:
    - cargo run --bin jlox 

run-clox:
    - cargo run --bin clox 