#!/bin/bash -ex

sh -c "$(curl --location https://taskfile.dev/install.sh)" -- -d -b ~/.local/bin
# shellcheck disable=SC2016
echo 'export PATH="$PATH:$HOME/.local/bin"' >> ~/.bashrc
