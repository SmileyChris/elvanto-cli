# Getting started

This page takes you from "I have a clone of the repo" to "I'm running my first
command against the live Elvanto API".

## Prerequisites

- Rust toolchain (stable, 1.75+). Install via [rustup](https://rustup.rs).
- An Elvanto account with an API key. In Elvanto: **Settings → Account →
  Integrations → API**. Copy the key — you will not be able to see it again.

## Build

From the repo root:

```sh
cargo build --release
```

The binary is at `target/release/elvanto`. Either add `target/release` to your
`PATH`, copy `elvanto` to `~/.local/bin`, or use the full path below.

## Provide an API key

`elvanto-cli` looks for an API key in three places, in this order:

1. The `ELVANTO_API_KEY` environment variable.
2. A `.env` file in the current directory or any parent.
3. The OS keyring (managed via `elvanto auth login` and `elvanto auth clear`).

See [Authentication](concepts/auth.md) for the full resolution rules.

The recommended setup for an interactive workstation is the keyring:

```sh
elvanto auth login           # prompts for the key, stores it via the OS keyring
elvanto auth status          # confirms source + verifies against Elvanto
```

For CI or scripts, set the environment variable directly:

```sh
export ELVANTO_API_KEY="your-key-here"
```

## First command

```sh
elvanto auth status
```

Expected output:

```
source: env (ELVANTO_API_KEY)
key:    abcd…wxyz
status: ok
```

If you see `status: invalid` or `source: none`, the message tells you what to
fix.

## Browse your data

```sh
elvanto songs categories            # list song categories
elvanto songs list --used-within 6m # songs used in the last 6 months
elvanto services list               # services in the last 6 months
elvanto people list                 # everyone active
elvanto people departments          # full department / sub-department / position tree
```

Every list command supports `--json` for structured output. Every list
command also accepts `--id short|long|hidden` to switch the id column
between short ids (default), full UUIDs, or omit the column entirely.

## Project-local defaults

Create a `.env` next to your repo or working directory:

```dotenv
ELVANTO_API_KEY=your-key-here

ELVANTO_SONGS_LIST="--category 45d9abe5 --used-within 6m --last-used"
ELVANTO_SERVICES_PEOPLE="--hide-unfilled --email"
```

Now `elvanto songs list` automatically applies those flags, but
`elvanto songs list --json` (any manual flag) or `--no-env` overrides them.
See [Auto-injected flags](concepts/env-flags.md) for the full rules.
