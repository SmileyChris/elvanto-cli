# elvanto-cli

`elvanto-cli` is a planned Rust command-line interface for the Elvanto API. Its
first-class use case is scriptable access to the Songs system, with output that
is useful both to humans at a terminal and to programs in shell pipelines.

The V1 read-only Songs surface (`auth check`, `songs categories`, `songs list`,
`songs show`, `songs chart`, `songs lyrics`) is implemented. The "Proposed
Command Shape" section below describes the original design surface, including
mutation commands that are deferred beyond V1; see [docs/songs.md](docs/songs.md)
for the actually-implemented V1 surface.

## Goals

- Provide predictable access to Elvanto song data from scripts, terminals, and
  CI jobs.
- Make common worship administration workflows faster: search songs, inspect
  arrangements, retrieve lyrics or chord charts, and create or update song
  metadata.
- Prefer stable machine output by defaulting automation to JSON while keeping
  concise text and table views for direct CLI use.
- Keep authentication, pagination, errors, and API request formatting in one
  shared Rust client instead of duplicating curl commands across scripts.

## Non-goals

- Replacing the Elvanto web UI.
- Covering every Elvanto API area before the songs workflow is reliable.
- Storing credentials in source-controlled files.
- Providing a daemon or long-running sync service in the initial release.

## API Basis

Elvanto exposes versioned endpoints under:

```text
https://api.elvanto.com/v1/SOME/METHOD.json
```

For the initial implementation, the CLI should call JSON endpoints with POSTed
JSON request bodies. Elvanto also documents XML, PHP, form-style POST, and query
parameter alternatives, but JSON POST keeps the Rust client and output contract
simple.

Authentication should initially support API-key authentication over HTTP Basic
Auth, using the API key as the username and a dummy password. OAuth 2 can be
added later for users who want delegated access; the relevant OAuth permission
for this project is `ManageSongs`.

Recommended local configuration:

```sh
export ELVANTO_API_KEY="..."
```

For local development, the CLI also loads a `.env` file from the current
directory or a parent directory:

```dotenv
ELVANTO_API_KEY=...
```

Future config file location:

```text
~/.config/elvanto-cli/config.toml
```

The environment variable should take precedence over config file values so
automation can inject secrets without modifying local files.

## Proposed Command Shape

The installed binary should be named `elvanto` even if the crate remains named
`elvanto-cli`.

```sh
elvanto auth check
elvanto songs list --title "great" --output table
elvanto songs list --artist "Chris Tomlin" --output json
elvanto songs show <song-id> --files
elvanto songs arrangements list <song-id> --transpose F#
elvanto songs arrangements show <arrangement-id> --transpose G
elvanto songs keys list <arrangement-id>
elvanto songs create --title "New Song" --arrangement "Default" --key C
elvanto songs edit <song-id> --title "Updated Title"
```

Global flags:

```text
--output text|table|json
--api-key <key>
--config <path>
--base-url <url>
--page <number>
--page-size <number>
--verbose
```

`--output json` should be the stable programmatic contract. Text and table
output can evolve to improve readability, but JSON field names should remain
stable unless a breaking release is explicitly documented.

## Songs Workflow Priority

1. Read-only discovery:
   - `songs list`
   - `songs show`
   - `songs arrangements list`
   - `songs arrangements show`
   - `songs keys list`
   - `songs keys show`

2. Human-friendly retrieval:
   - print lyrics
   - print chord charts
   - request a transposed chord chart when Elvanto supports it
   - include files only when requested

3. Mutations:
   - create a song with at least one arrangement
   - edit song metadata
   - create and edit arrangements
   - create and edit keys

## Output Principles

- Default terminal output should be compact and readable.
- JSON output should preserve the useful API fields while normalizing obvious
  Rust-side names like `song_id`, `arrangement_id`, and `key_starting`.
- Commands should exit non-zero on Elvanto API failures and print actionable
  error messages to stderr.
- Empty search results should be distinct from transport or authentication
  failures.
- Pagination should be explicit by default, with a later `--all` mode for
  fetching every page when useful.

## Suggested Rust Stack

- `clap` for command parsing.
- `reqwest` for HTTP.
- `serde` and `serde_json` for request and response types.
- `tokio` for async execution.
- `thiserror` for typed application errors.
- `directories` for config path discovery.
- `comfy-table` or `tabled` for text tables.
- `secrecy` for in-memory handling of credentials where practical.

## Documentation

- [Songs system design](docs/songs.md)

## Building & running

The crate ships as a binary called `elvanto`.

````sh
cargo build --release
export ELVANTO_API_KEY="your-key"
./target/release/elvanto auth check
./target/release/elvanto songs categories
./target/release/elvanto songs list --album --ccli --category-id "short-or-full-category-id" --used-within 6m --not-used-within 2w
./target/release/elvanto services list
````

For local development against a stub server:

````sh
ELVANTO_BASE_URL=http://localhost:8080 cargo run -- songs list
````

## References

API facts in these initial docs were checked against Elvanto's official
documentation on 2026-05-19:

- [Elvanto API: Getting Started](https://www.elvanto.com/api/getting-started/)
- [Elvanto API: Songs](https://www.elvanto.com/api/songs/)
- [Elvanto API: songs/getAll](https://www.elvanto.com/api/songs/getAll/)
- [Elvanto API: songs/create](https://www.elvanto.com/api/songs/create/)
- [Elvanto API: songs/arrangements/getAll](https://www.elvanto.com/api/songs/arrangements/getAll/)
- [Elvanto API: songs/keys/getAll](https://www.elvanto.com/api/songs/keys/getAll/)
