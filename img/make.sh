#!/bin/bash
set -e
cd "$(dirname "${BASH_SOURCE[0]}")"
HOST=x86_64-unknown-linux-gnu
TARGET=m68k-none-eabi
LLVM_BIN=$(realpath ../../llvm/build-relwithdebinfo/bin)
RUSTDIR=$(realpath ../../rust/build/$HOST)
export RUSTC=$RUSTDIR/stage2/bin/rustc
CARGO=$RUSTDIR/stage2-tools-bin/cargo
CLANG=$LLVM_BIN/clang
LD=$LLVM_BIN/ld.lld

pushd ..
$CARGO -Z build-std=core build --target .cargo/$TARGET.json -v
popd
$CLANG -target $TARGET -c entry.S
$LD -o img.elf -Ttext=0 entry.o ../target/$TARGET/debug/libtest_project.rlib
