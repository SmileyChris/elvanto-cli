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
- **IDs, names, or paths — your call.** Elvanto returns full UUIDs;
  `elvanto-cli` accepts the full UUID, its short first-block, or a name /
  path like `Music Team/Vocals`. Ambiguous matches fail with a disambiguation
  table; typos get fuzzy suggestions. See [Lookups](concepts/lookups.md).
- **Default flags via environment variables.** Set `ELVANTO_SONGS_LIST`,
  `ELVANTO_SERVICES_PEOPLE`, etc. in `.env` and they auto-apply to that
  subcommand. See [Auto-injected flags](concepts/env-flags.md).
- **Consistent filter flag across commands.** `--in` works the same on
  `people list` and `services people` — isolate worship leaders, vocalists,
  or production crew with one syntax.

## Where to next

- Brand new? Start with [Getting started](getting-started.md).
- Want to know how the API key gets resolved? See [Authentication](concepts/auth.md).
- Looking for a command's flags? Jump into the [Commands](commands/auth.md) section.
