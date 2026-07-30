default: deploy

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

audit:
    cargo audit

logs:
    journalctl --user -u afanasieff -f

db:
    sqlite3 ~/.local/state/afanasieff/afanasieff.db
