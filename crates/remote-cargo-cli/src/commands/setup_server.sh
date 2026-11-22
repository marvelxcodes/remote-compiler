set -e
architecture="$(uname -m)"
if [ "$architecture" != x86_64 ]; then
    echo "server architecture is $architecture, binaries would not run on an x86_64 client" >&2
    exit 1
fi
packages="build-essential pkg-config libssl-dev rsync curl"
if ! dpkg -s $packages >/dev/null 2>&1; then
    sudo apt-get update
    sudo apt-get install -y $packages
fi
if [ ! -f "$HOME/.cargo/env" ]; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
fi
. "$HOME/.cargo/env"
rustup update stable --no-self-update
cargo --version
rsync --version | head -n 1
