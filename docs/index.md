# elvanto-cli

`elvanto-cli` is a Rust command-line interface for the [Elvanto](https://www.elvanto.com)
API. It is built for scriptable, predictable access to song, service, and
people data — equally usable at a terminal and from automation.

The installed binary is named `elvanto`.

## What you get

- **Auth that fits the way you actually work** — `ELVANTO_API_KEY` from the
  environment, a project-local `.env` file, or a key stored in the OS keyring.
- **Stable JSON output** behind `--json` on every list command, with normalized
  field names. Text output is compact tables suitable for piping into `awk`,
  `cut`, or `fzf`.
- **Short IDs everywhere.** Elvanto returns full UUIDs, but the first
  dash-separated block is unique within each list — so `elvanto songs list`
  shows (and accepts) `45d9abe5` instead of `45d9abe5-1234-5678-9abc-...`.
- **Default flags via environment variables.** Set `ELVANTO_SONGS_LIST`,
  `ELVANTO_SERVICES_PEOPLE`, etc. in `.env` and they auto-apply to that
  subcommand. See [Auto-injected flags](concepts/env-flags.md).
- **Department / position filtering by id** — work the same way on both
  `people list` and `services people` so you can isolate worship leaders,
  vocalists, or production crew with one consistent flag.

## Where to next

- Brand new? Start with [Getting started](getting-started.md).
- Want to know how the API key gets resolved? See [Authentication](concepts/auth.md).
- Looking for a command's flags? Jump into the [Commands](commands/auth.md) section.
