#!/bin/bash

# shellcheck disable=SC2164

set -e

if [[ -z "$SRC_PATH" ]]; then
    echo "SRC_PATH is not set or empty"
    exit 1
fi

# Adapted from https://raw.githubusercontent.com/snapcore/snapcraft/master/docker/Dockerfile
SNAP_ARCH="Unknown"
if [[ "$(arch)" == "x86_64" ]]; then SNAP_ARCH="amd64"; fi
if [[ "$(arch)" == "aarch64" ]]; then SNAP_ARCH="arm64"; fi

function install-snap() {
  SNAP_NAME=$1

  echo "Installing $SNAP_NAME..."

  curl -L $(curl -H 'X-Ubuntu-Series: 16' -H "X-Ubuntu-Architecture: $SNAP_ARCH" "https://api.snapcraft.io/api/v1/snaps/details/$SNAP_NAME" | jq '.download_url' -r) --output $SNAP_NAME.snap

  mkdir -pv /snap/$SNAP_NAME
  unsquashfs -d /snap/$SNAP_NAME/current $SNAP_NAME.snap || true

  rm $SNAP_NAME.snap
}

export HOME=/root
export TERM=xterm
export PATH="/root/.local/bin:/usr/local/bin:/usr/local/sbin:/usr/bin:/usr/sbin:/bin:/sbin:/usr/lib/gcc/$(arch)-linux-gnu/9"
export LD_LIBRARY_PATH="/usr/lib/gcc/$(arch)-linux-gnu/9"
export SNAPCRAFT_VERSION="8.9.1"

apt-get update

ln -sf /usr/share/zoneinfo/Etc/UTC /etc/localtime
DEBIAN_FRONTEND=noninteractive apt-get install -y tzdata
dpkg-reconfigure --frontend noninteractive tzdata

apt-get install -y sudo locales curl ca-certificates jq squashfs-tools build-essential git python3-pygit2 python3-pip libxml2-dev libxslt1-dev python3-venv libapt-pkg-dev libgit2-dev cargo pkg-config libssl-dev libyaml-dev xdelta3 python3-apt execstack
locale-gen en_US.UTF-8

cd
git clone https://github.com/canonical/snapcraft.git
cd snapcraft && git checkout $SNAPCRAFT_VERSION && git submodule update --init --recursive
make setup
uv build
cd dist && pip install ./snapcraft-$SNAPCRAFT_VERSION-py3-none-any.whl --break-system-packages
cd

# Adapted from https://raw.githubusercontent.com/snapcore/snapcraft/master/docker/Dockerfile
apt-get install -y snapd
install-snap core20
install-snap core24
install-snap gtk-common-themes
install-snap gnome-3-38-2004

export LANG="en_US.UTF-8"
export LANGUAGE="en_US:en"
export LC_ALL="en_US.UTF-8"
export PATH="/snap/bin:$PATH"
export SNAP_ARCH="$(arch)"

cd $SRC_PATH && snapcraft
