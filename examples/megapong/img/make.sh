#!/bin/bash
set -e
cd "$(dirname "${BASH_SOURCE[0]}")"

[ -z "$LLVM_CONFIG" ] && LLVM_CONFIG=$(which llvm-config)

TARGET=m68k-none-eabi
LLVM_BIN=$("$LLVM_CONFIG" --bindir)
CLANG=$LLVM_BIN/clang
LD=$LLVM_BIN/ld.lld
OBJCOPY=$LLVM_BIN/llvm-objcopy
TARGET_OUT=../../../target/$TARGET/release

pushd ..
cargo -Z build-std=core build --release --target .cargo/$TARGET.json
popd
$CLANG -target $TARGET -c entry.S
$LD --gc-sections -o img.elf -Ttext=0 entry.o $TARGET_OUT/libmegapong.rlib $TARGET_OUT/deps/libcore-*.rlib $TARGET_OUT/deps/libcompiler_builtins-*.rlib
$OBJCOPY -O binary img.elf img.md
