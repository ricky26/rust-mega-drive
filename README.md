# Mega Drive toolkit for Rust

This repository hosts a few packages for use with the SEGA Mega Drive.

## Using these packages
At the moment, these packages only work with a fork of LLVM & Rust. To use them
you will need to build both.

### Building LLVM
1. Checkout the `llvm-12` branch from
[LLVM project fork](https://github.com/ricky26/M680x0-mono-repo).
  
2. Build the toolchain with cmake:
    1. Generate the project with the M68k backend enabled:
       ```bash
       cd M68x0-mono-repo
       mkdir build
       cd build
       cmake -G Ninja "-DLLVM_USE_LINKER=lld" "-DCMAKE_BUILD_TYPE=RelWithDebInfo" "-DLLVM_ENABLE_ASSERTIONS=ON" "-DLLVM_PARALLEL_LINK_JOBS=1" "-DLLVM_TARGETS_TO_BUILD=X86" "-DLLVM_EXPERIMENTAL_TARGETS_TO_BUILD=M68k" "-DLLVM_ENABLE_PROJECTS=clang;lld" ..
       ```
    2. Build:
       ```
       ninja -j16
       ```
       (This step can take along time and a lot of memory if used with a lot of threads.)
    3. You should now have all of the LLVM binaries in `build/bin`.
    
### Building Rust
1. Checkout the [Rust fork](https://github.com/ricky26/rust) (clone the
   m68k-linux branch).
   
2. Copy `config.toml.example` to `config.toml` and edit:
    1. Set `[build] target = ["x86_64-unknown-linux-gnu", "m68k-unknown-linux-gnu"`
    2. Set `[target.x86_64-unknown-linux-gnu] llvm-config = "path/to/build/bin/llvm-config"`
    
3. Build:
    ```
    ./x.py build --stage=2 rustc cargo
    ```
4. You should now have a Rust toolchain in `build/x86_64-unknown-linux-gnu/stage2`.
5. Link the toolchain in rustup so it is easier to use:
    ```
    rustup toolchain link m68k "path/to/build/x86_64-unknown-linux-gnu/stage2"
    ```

### Building this repository
1. Set the required environment variables:
    ```
    export MEGADRIVE_TARGET=path/to/repo/targets/m68k-none-eabi.json
    export RUSTUP_TOOLCHAIN=m68k
    export LLVM_CONFIG=path=/to/llvm/build/bin/llvm-config
    ```
2. Build the tools & libraries:
    ```
    cargo build --release
    ```
3. Install the cargo tool:
    ```
    cd tools/cargo-megadrive
    cargo install --path=.
    ```
4. Build the example Mega Drive image:
    ```
    cd examples/megapong
    cargo megadrive build
    ```
5. You should now have an example megadrive image in
    `target/m68k-none-eabi/release/megapong.md`.