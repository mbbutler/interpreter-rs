check:
    @- echo "Linting files..."
    - cargo fmt -- --check
    - cargo clippy -- -D warnings

test:
    - cargo test