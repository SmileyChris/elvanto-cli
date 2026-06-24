# elvanto-cli

A Rust CLI for the [Elvanto](https://elvanto.com) API — scriptable access to
songs, services, and people data.

```sh
elvanto auth login              # store API key in OS keyring
elvanto songs list --last-used  # active songs, most recent first
elvanto songs show <id>         # full song details
elvanto songs chart <id>        # chord chart
elvanto songs lyrics <id>       # lyrics
elvanto services list           # upcoming & recent services
elvanto services people <id>    # who's rostered
elvanto services song-usage     # songs used 1-2 times, with leaders
elvanto people list             # active people
```

Every command supports `--json` for structured output and `--help` for full
flag docs.

## Install

Pre-built binaries for Linux, macOS, and Windows on the [releases
page](https://github.com/SmileyChris/elvanto-cli/releases). See
[`docs/getting-started.md`](docs/getting-started.md) for building from source
and auth setup.

## Docs

[`docs/`](docs/) — full user docs (built with [Zensical](https://zensical.org)).

## License

MIT
