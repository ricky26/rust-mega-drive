# Use custom built Rust Motorola 68000 targeted base image
FROM quay.io/reinvantveer/rust-m68k-megadrive:1.53.0-dev
MAINTAINER rickytaylor26@gmail.com
MAINTAINER rein@vantveer.me

# Install Python3 and pip in order to run the build script for the font generator
RUN apt-get update && apt-get install -y --no-install-recommends python3-pip && rm -rf /var/lib/apt/lists/*

# Copy over all the files
COPY . /rust-mega-drive

ENV MEGADRIVE_HOME=/rust-mega-drive/share
ENV RUSTUP_TOOLCHAIN=m68k
ENV LLVM_CONFIG=/llvm-m68k/bin/llvm-config

# Build pong example
WORKDIR /rust-mega-drive/examples/megapong
RUN cargo megadrive --verbose build

# Build coin flip example
WORKDIR /rust-mega-drive/examples/megacoinflip
RUN cargo megadrive --verbose build

# For now: copy at runtime the compiled target files to a /target dir that can be mounted using docker run -v
CMD ["cp", "-r", "/rust-mega-drive/target", "/target"]
