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
- **Hands out achievements.** It counts how people behave in the chat — night
  posting, monologues, unanswered calls, who tags whom — and drops an
  achievement as a reply the moment one is earned, in Matthew's own voice.
  `/achievements` lists all seventeen with their conditions, `/my_achievements`
  shows what the caller has unlocked and what is still locked. Counters start
  from zero when the bot first runs this version; there is no history behind
  them. `/top` ranks the whole chat by how many achievements each member has
  collected.
- **Listens to reactions.** A reaction pushes the message it lands on into the
  quote pool ahead of the queue, makes a quote come up more often, and counts as
  an answer for the achievements. It needs the bot to be an administrator of the
  group — Telegram sends reaction updates to nobody else.
- **Talks nonsense.** A Markov chain over everything it has collected produces
  phrases nobody actually said. `/bred` asks for one; a quarter of the hourly
  messages and a fifth of the keyword replies are generated rather than quoted.
- **Picks a cuckold of the day.** The first `/cuckold` of the day draws one
  member at random out of everyone the bot has seen in the last thirty days,
  announces them with a drum roll — one message edited twice — in Matthew's
  voice, and counts it; every later call that day repeats the same name. The
  day is a Moscow day, so an evening and the small hours after it count as
  one. Runs of consecutive days are tracked, and the longest is kept.
  `/cuckold_stats` ranks the chat by how often each member has been drawn,
  with medals for the top three, the longest run beside anyone who has one,
  and a line naming today's pick once the day has been drawn.

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
`cargo-tarpaulin`, `rainfrog`). It knows `pacman`, `apt`, `dnf` and `brew`, and asks for `sudo`
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
| `just coverage` | Test coverage against the 70% floor |
| `just audit` | Dependency vulnerability scan |
| `just logs` | Follow the running service's logs |
| `just db` / `just db-tui` | Open the quote database in a shell or a TUI |

## Status

A small personal project, written in Rust, running for one group chat. The
quote pool is in-jokes and is not meant to be read out of context.

## License

MIT License — see [LICENSE](./LICENSE).

Copyright (c) 2024 Ivan Ivanchuk
