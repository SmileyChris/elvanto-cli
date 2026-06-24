# Getting started

This page takes you from zero to "I'm running my first command against the
live Elvanto API".

## Prerequisites

- An Elvanto account with an API key. In Elvanto: **Settings → Account →
  Integrations → API**. Copy the key — you will not be able to see it again.

## Install

Two options. Use the pre-built binary if you just want to run `elvanto`;
build from source if you want to hack on it.

### Option 1: pre-built binary (recommended)

Pre-built archives for Linux, macOS (arm64 + x86_64), and Windows are
published on each [release](https://github.com/SmileyChris/elvanto-cli/releases).
Pick the archive for your platform from the latest release, extract it,
and drop the `elvanto` binary somewhere on your `PATH`.

```sh
# Linux x86_64
curl -L "https://github.com/SmileyChris/elvanto-cli/releases/latest/download/elvanto-x86_64-unknown-linux-gnu.tar.gz" \
  | tar -xz -C ~/.local/bin

# macOS arm64 (Apple Silicon)
curl -L "https://github.com/SmileyChris/elvanto-cli/releases/latest/download/elvanto-aarch64-apple-darwin.tar.gz" \
  | tar -xz -C ~/.local/bin

# macOS x86_64 (Intel)
curl -L "https://github.com/SmileyChris/elvanto-cli/releases/latest/download/elvanto-x86_64-apple-darwin.tar.gz" \
  | tar -xz -C ~/.local/bin

# Windows x86_64 (use the .zip from the releases page)
```

```sh
elvanto --version
```

### Option 2: build from source

Needs a Rust toolchain (stable, 1.75+). Install via [rustup](https://rustup.rs).

```sh
git clone https://github.com/SmileyChris/elvanto-cli.git
cd elvanto-cli
cargo build --release
```

The binary lands at `target/release/elvanto`. Either add `target/release`
to your `PATH`, copy `elvanto` to `~/.local/bin`, or use the full path
below.

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
elvanto people org                  # full department / sub-department / position tree
```

Every list command supports `--json` for structured output. Every list
command also accepts `--id short|long|hidden` to switch the id column
between short ids (default), full UUIDs, or omit the column entirely.

Flags that take an entity reference (`--in`, `--category`, `--arrangement`,
and the `<ID>` positional args) accept ids **or** names/paths:

```sh
elvanto people list --in "Music Team/Vocals"
elvanto songs list --category Contemporary
elvanto services people 02b06b47 --in "Worship Leader"
```

See [Lookups](concepts/lookups.md) for the full rules.

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
