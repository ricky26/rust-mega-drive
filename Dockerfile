FROM ubuntu:20.04 AS llvm-builder
MAINTAINER rein@vantveer.me
ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        git \
        curl \
        ca-certificates \
        build-essential \
        cmake \
        python3 \
        lld \
        ninja-build

RUN git clone -b llvm-12 --single-branch https://github.com/ricky26/llvm-project /llvm-project

# Lowering the number of jobs may help you solve out of memory crashes
# Increasing the number of jobs will save you time :)
ARG NUM_JOBS=5
WORKDIR /llvm-m68k
RUN cmake \
    -S /llvm-project/llvm \
    -G Ninja \
    "-DLLVM_USE_LINKER=lld" \
    "-DCMAKE_BUILD_TYPE=RelWithDebInfo" \
    "-DLLVM_ENABLE_ASSERTIONS=ON" \
    "-DLLVM_PARALLEL_LINK_JOBS=1" \
    "-DLLVM_TARGETS_TO_BUILD=X86" \
    "-DLLVM_EXPERIMENTAL_TARGETS_TO_BUILD=M68k" \
    "-DLLVM_ENABLE_PROJECTS=clang;lld" \
    /llvm-m68k \
    && cmake --build . --parallel ${NUM_JOBS} \
    && cmake --build . --target install \
    && rm -rf /llvm-project

WORKDIR /rust-m68k
RUN git clone -b m68k-linux --single-branch https://github.com/ricky26/rust.git /rust-m68k
RUN cp config.toml.example config.toml
RUN sed -i 's|#target = \["x86_64-unknown-linux-gnu"\]|target = \["x86_64-unknown-linux-gnu", "m68k-unknown-gnu"\]|g' config.toml
RUN grep 'target = \["x86_64-unknown-linux-gnu", "m68k-unknown-gnu"\]' config.toml
RUN sed -i 's|#llvm-config = "../path/to/llvm/root/bin/llvm-config"|llvm-config = "/llvm-m68k/bin/llvm-config"|g' config.toml
RUN grep 'llvm-config = "/llvm-m68k/bin/llvm-config"' config.toml

RUN python3 x.py build --stage=2 rustc cargo
RUN rustup toolchain link m68k "build/x86_64-unknown-linux-gnu/stage2"
