default: deploy

build:
    cargo build --release

restart:
    systemctl --user restart afanasieff

deploy: build restart
    systemctl --user status afanasieff --no-pager

logs:
    journalctl --user -u afanasieff -f
