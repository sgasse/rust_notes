# Rust Notes

Some notes about language concepts, snippets etc.

- [`GstReader example`](./gst_reader/)
- [`macro_rules!` examples](./macro_examples/)
- [`Send` and `Sync`](./send_sync/README.md)
- [Stream modulation](./stream_modulation/)

## Links

- [Implement `serde::Deserialize`](https://stackoverflow.com/a/46755370)
- [Demystifying trait generics](https://gruebelinchen.wordpress.com/2023/06/06/demystifying-trait-generics-in-rust/)
- [`bilge` for bit fiddling](https://hecatia-elegua.github.io/blog/no-more-bit-fiddling/)
- [Scaling Rust builds with Bazel](https://mmapped.blog/posts/17-scaling-rust-builds-with-bazel.html)
- [Pin and Unpin](https://blog.cloudflare.com/pin-and-unpin-in-rust/)
- [Sharing data among embassy tasks](https://dev.to/apollolabsbin/sharing-data-among-tasks-in-rust-embassy-synchronization-primitives-59hk)

## Cargo

See fingerprint info during cargo build

```bash
CARGO_LOG=cargo::core::compiler::fingerprint=info cargo build
```
