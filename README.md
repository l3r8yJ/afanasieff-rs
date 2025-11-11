# Afanasieff-rs

A Rust-based Telegram bot that delivers inspirational quotes from various sources.

## Features

- **Multiple Quote Sources**: Delivers quotes from Matthew, Vinograd, and Stream sources
- **Scheduled Delivery**: Automatically sends hourly quotes via cron job
- **Telegram Integration**: Built with the elegant [teloxide](https://github.com/teloxide/teloxide) framework
- **Async/Await**: Fully asynchronous design using Tokio runtime

## Quote Sources

- **Matthew**: Inspirational quotes from Matthew
- **Vinograd**: Wisdom from Vinograd
- **Stream**: Dynamic stream-based quotes

## Building

```bash
cargo build --release
```

## Running

Set your Telegram bot token as an environment variable:

```bash
export TELOXIDE_TOKEN="your_bot_token_here"
cargo run
```

## Requirements

- Rust 1.82+

## Development

Run clippy for linting:

```bash
cargo clippy -- -D clippy::all -D clippy::pedantic -D clippy::nursery -D clippy::cargo -D warnings
```

Run tests:

```bash
cargo test
```

## License

MIT License - See [LICENSE](./LICENSE) file for details

Copyright (c) 2024 Ivan Ivanchuk
