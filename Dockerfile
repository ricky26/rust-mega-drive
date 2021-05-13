FROM rust-m68k:latest

# Build the rust-mega-drive crate
COPY . /rust-mega-drive
WORKDIR /rust-mega-drive
ENV MEGADRIVE_HOME=/rust-mega-drive
ENV RUSTUP_TOOLCHAIN=m68k
ENV LLVM_CONFIG=/llvm-m68k/bin/llvm-config
RUN $HOME/.cargo/bin/cargo build --release

# Install the megadrive cargo command
WORKDIR /rust-mega-drive/tools/cargo-megadrive
RUN $HOME/.cargo/bin/cargo install --path=.

# Build megapong
WORKDIR /rust-mega-drive/examples/megapong
RUN $HOME/.cargo/bin/cargo megadrive build
