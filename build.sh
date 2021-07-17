#!/usr/bin/env bash
set -eux

# Check whether any cli args were supplied
if [[ "$#" -ne 0 ]]; then
  # If so, then if the arg is 'rm' then throw away the previously built target
  if [[ $1 == 'rm' ]]; then
    rm -rf target
  fi
fi

docker build -t rust-mega-drive:latest .
docker run -it -v $(pwd)/target:/target rust-mega-drive:latest
sudo chown -R $USER:$USER target
