FROM rust-mega-drive:latest
MAINTAINER rickytaylor26@gmail.com
MAINTAINER rein@vantveer.me

# Copy over all files
COPY . /rust-mega-drive

# Build pong example
WORKDIR /rust-mega-drive/examples/megapong
RUN cargo megadrive --verbose build

# Build coin flip example
WORKDIR /rust-mega-drive/examples/megacoinflip
RUN cargo megadrive --verbose build

# For now: copy at runtime the compiled target files to a /target dir that can be mounted using docker run -v
CMD ["cp", "-r", "/rust-mega-drive/target", "/target"]
