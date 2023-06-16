# Rust Notes

Some notes about language concepts, snippets etc.

- [`GstReader example`](./gst_reader/)
- [`macro_rules!` examples](./macro_examples/)
- [`Send` and `Sync`](./send_sync/README.md)
- [Stream modulation](./stream_modulation/)

## Cargo

See fingerprint info during cargo build

```bash
CARGO_LOG=cargo::core::compiler::fingerprint=info cargo build
```
