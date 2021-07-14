#!/usr/bin/env bash
set -eux pipefail
docker build -t rust-mega-drive:latest .
docker run -it -v $(pwd)/target:/target rust-mega-drive:latest
sudo chown -R $USER:$USER target
