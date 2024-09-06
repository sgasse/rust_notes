# Manual FFI setup to call Rust from C

Build

```sh
cargo build
gcc call_cffi.c -o call_cffi -lcffi -L./target/debug

```

Execute

```sh
LD_LIBRARY_PATH=./target/debug ./call_cffi
```

Resources:

- [Rustonomicon: FFI](https://doc.rust-lang.org/nomicon/ffi.html)
- [FFI Omnibus - Objects by Jake Goulding](http://jakegoulding.com/rust-ffi-omnibus/objects/)
- [Complex data types & Rust FFI by Kyle Douglass](http://kmdouglass.github.io/posts/complex-data-types-and-the-rust-ffi/)
