default: deploy

setup-env:
    #!/usr/bin/env bash
    set -euo pipefail
    if command -v pacman >/dev/null 2>&1; then
        sudo pacman -S --needed --noconfirm base-devel perl sqlite rustup
    elif command -v apt-get >/dev/null 2>&1; then
        sudo apt-get update
        sudo apt-get install -y build-essential perl sqlite3 curl
    elif command -v dnf >/dev/null 2>&1; then
        sudo dnf install -y gcc make perl sqlite curl
    elif command -v brew >/dev/null 2>&1; then
        brew install sqlite rustup
    else
        echo "unknown package manager: install a C toolchain, perl and sqlite by hand" >&2
        exit 1
    fi
    if ! command -v rustup >/dev/null 2>&1; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    fi
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi
    rustup toolchain install stable --component rustfmt clippy
    cargo install --locked cargo-audit cargo-tarpaulin rainfrog

build:
    cargo build --release

restart:
    systemctl --user restart afanasieff

deploy: check build restart
    systemctl --user status afanasieff --no-pager

check: fmt clippy test doc

fmt:
    cargo fmt -- --check

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

test:
    cargo test --all-features

doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps

coverage:
    cargo tarpaulin --all-features --exclude-files "src/main.rs" --fail-under 70

audit:
    cargo audit

logs:
    journalctl --user -u afanasieff -f

db:
    sqlite3 ~/.local/state/afanasieff/afanasieff.db

db-tui:
    rainfrog --url sqlite://$HOME/.local/state/afanasieff/afanasieff.db
