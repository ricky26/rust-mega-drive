FROM rust-m68k:latest
MAINTAINER rickytaylor26@gmail.com
MAINTAINER rein@vantveer.me

# Copy over all files
COPY . /rust-mega-drive

# Build the rust-mega-drive crate
WORKDIR /rust-mega-drive
ENV MEGADRIVE_HOME=/rust-mega-drive/share
ENV RUSTUP_TOOLCHAIN=m68k
ENV LLVM_CONFIG=/llvm-m68k/bin/llvm-config
RUN cargo build --release

# Install the megadrive cargo command
WORKDIR /rust-mega-drive/tools/cargo-megadrive
RUN cargo install --path=.

# Build megapong as default command
WORKDIR /rust-mega-drive/examples/megapong
RUN cargo megadrive --verbose build

WORKDIR /rust-mega-drive/examples/megacoinflip
RUN cargo megadrive --verbose build

# For now: copy at runtime the compiled target files to a /target dir that can be mounted using docker run -v
CMD ["cp", "-r", "/rust-mega-drive/target", "/target"]
