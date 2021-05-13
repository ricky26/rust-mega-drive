FROM rust-m68k:latest
MAINTAINER rickytaylor26@gmail.com
MAINTAINER rein@vantveer.me

# Build the rust-mega-drive crate
COPY . /rust-mega-drive
WORKDIR /rust-mega-drive
ENV MEGADRIVE_HOME=/rust-mega-drive
ENV RUSTUP_TOOLCHAIN=m68k
ENV LLVM_CONFIG=/llvm-m68k/bin/llvm-config
RUN cargo build --release

# Install the megadrive cargo command
WORKDIR /rust-mega-drive/tools/cargo-megadrive
RUN cargo install --path=.

# Build megapong as default command
WORKDIR /rust-mega-drive/examples/megapong
CMD ["cargo", "megadrive", "--verbose", "build"]
