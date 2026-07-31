# Afanasieff-rs

A Telegram chat bot that keeps a group's own quotes alive and throws them back
at the chat.

## What it does

- **Reacts to keywords.** When someone mentions one of the tracked names, the
  bot replies with a random quote attributed to that person and reacts to the
  message.
- **Speaks up on its own.** Every so often it picks a chat it knows and drops a
  quote there unprompted.
- **Learns from the chat.** Messages worth remembering are picked up as they
  arrive and gradually become part of the quote pool, so the bot's material
  grows with the conversation.

## Quote sources

Quotes are grouped by who said them — `matthew`, `stream` and `vinograd`. Each
source has its own keyword and its own reaction emoji.

## Getting started

Install [`just`](https://github.com/casey/just), then let it do the rest:

```
just setup-env
```

That brings in everything the repo needs — a C toolchain and `perl` for the
dependencies that build from source, the Rust toolchain with `rustfmt` and
`clippy`, `sqlite3`, and the extra tools some recipes call (`cargo-audit`,
`rainfrog`). It knows `pacman`, `apt`, `dnf` and `brew`, and asks for `sudo`
where the package manager needs it.

The one thing it cannot fetch for you is a Telegram bot token: get one from
[@BotFather](https://t.me/BotFather) and export it as `TELOXIDE_TOKEN` before
the bot starts.

Deploy and log recipes additionally expect a `systemd` user unit named
`afanasieff`, which only matters on the machine that actually runs the bot.

## Working on it

Every routine task goes through [`just`](https://github.com/casey/just) — the
`justfile` is the entry point, `just --list` shows everything.

| Recipe | What it does |
| --- | --- |
| `just setup-env` | Install the toolchain and the tools the other recipes need |
| `just` | Full path: checks, build, restart the service, show its status |
| `just check` | Formatting, lints, tests and docs |
| `just build` | Release build |
| `just deploy` | Check, build and restart |
| `just audit` | Dependency vulnerability scan |
| `just logs` | Follow the running service's logs |
| `just db` / `just db-tui` | Open the quote database in a shell or a TUI |

## Status

A small personal project, written in Rust, running for one group chat. The
quote pool is in-jokes and is not meant to be read out of context.

## License

MIT License — see [LICENSE](./LICENSE).

Copyright (c) 2024 Ivan Ivanchuk
