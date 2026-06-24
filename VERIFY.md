# TSCP Anchor Verification

## Requirements
- Rust stable
- Cargo

## Verify
```bash
git verify-tag v0.1-event-algebra
cargo test --workspace
sha256sum -c proof-bundle/v0.1-event-algebra/files.sha256
\```
