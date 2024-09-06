cargo build
gcc call_cffi.c -o call_cffi -lcffi -L./target/debug
LD_LIBRARY_PATH=./target/debug ./call_cffi
