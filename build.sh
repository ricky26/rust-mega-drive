#!/usr/bin/env bash
set -eux

# Check whether any cli args were supplied
if [[ "$#" -ne 0 ]]; then
  # If so, then if the arg is 'rm' then throw away the previously built target
  if [[ $1 == 'rm' ]]; then
    rm -rf target
  fi
fi

# Build the crates in the `examples` folder
docker-compose build mega-drive

# Copy files to mounted target folder
docker-compose run --rm mega-drive "bash -c cp -r /rust-mega-drive/target /target"

# Re-assign "target" dir to current user
sudo chown -R $USER:$USER target
flatpak run org.libretro.RetroArch \
  --load-menu-on-error \
  -L ~/.var/app/org.libretro.RetroArch/config/retroarch/cores/blastem_libretro.so
