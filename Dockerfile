FROM ubuntu:20.04
ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        git \
        ca-certificates

RUN git clone -b llvm-12 --single-branch https://github.com/ricky26/llvm-project llvm-m68k
RUN mkdir -p llvm-m68k/build
WORKDIR llvm-m68k/build

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        cmake \
        python3 \
        lld \
        ninja-build
RUN cmake \
    -S ../llvm \
    -G Ninja \
    "-DLLVM_USE_LINKER=lld" \
    "-DCMAKE_BUILD_TYPE=RelWithDebInfo" \
    "-DLLVM_ENABLE_ASSERTIONS=ON" \
    "-DLLVM_PARALLEL_LINK_JOBS=1" \
    "-DLLVM_TARGETS_TO_BUILD=X86" \
    "-DLLVM_EXPERIMENTAL_TARGETS_TO_BUILD=M68k" \
    "-DLLVM_ENABLE_PROJECTS=clang;lld" \
    ..
RUN ninja -j8
