# Songs System Design

The songs module is the V1 focus for `elvanto-cli`. V1 is read-only;
mutations are deferred.

## V1 Command Surface

```
elvanto songs categories           [--json] [--full-id]
elvanto songs list                  [--json] [--album] [--ccli] [--category-id ID ...] [--full-id]
elvanto songs show <id>             [--json] [--full] [--files]
elvanto songs chart <id>            [--transpose KEY|OFFSET] [--arrangement NAME]
elvanto songs lyrics <id>           [--arrangement NAME]
```

See [CONTEXT.md](../CONTEXT.md) for terminology and output defaults.

## Endpoint Mapping

| CLI command | Elvanto method | Key parameters |
| --- | --- | --- |
| `songs categories` | `songs/categories/getAll` | none |
| `songs list` | `songs/getAll` | `page`, `page_size`, `item=0`; optional client-side `--category-id` filter |
| `songs show <id>` | `songs/getInfo` | `id`, `files` |
| `songs chart <id>` | `songs/arrangements/getAll` + `songs/arrangements/getInfo` | `song_id`, `chord_chart_key` |
| `songs lyrics <id>` | `songs/arrangements/getAll` + `songs/arrangements/getInfo` | `song_id` |

No separate arrangement or key subcommands. All arrangement/key data lives
inside `songs show`. `songs chart` and `songs lyrics` are convenience
commands that pick the Default arrangement (or the first if no Default) and
output human-readable text only.

## Command Details

### `songs categories`

Lists all song categories with id and name. Text output uses the first UUID
block as a short id by default; pass `--full-id` to print full UUIDs. JSON output
always keeps full ids.

```sh
elvanto songs categories
elvanto songs categories --full-id
elvanto songs categories --json
```

### `songs list`

Lists active songs. Default text output: `id | title | artist`, using first UUID
blocks as short song ids. `--full-id` prints full song UUIDs in text output.
`--album` and `--ccli` flags add those columns. `--category-id` keeps songs
assigned to that category id; it accepts either the full UUID or first-block
short id. Repeat it to OR-match multiple categories. `--json` returns all
matching songs (including non-active) as normalized JSON with full ids.
Auto-fetches all pages.

```sh
elvanto songs list
elvanto songs list --full-id
elvanto songs list --album --ccli
elvanto songs list --category-id abc --category-id def
elvanto songs list --json
```

### `songs show`

Curated output by default: title, artist, CCLI number, status, first line of
lyrics, arrangement/key summary. `--full` adds categories, locations, notes,
sequence, BPM, duration, learn/allow_downloads flags. `--json` returns the full
normalized song object. `--files` includes attachment data in JSON mode.

```sh
elvanto songs show 84937df8-e993-11e2-b739-a20c5589acc5
elvanto songs show 84937df8-e993-11e2-b739-a20c5589acc5 --full
elvanto songs show 84937df8-e993-11e2-b739-a20c5589acc5 --json --files
```

### `songs chart`

Dumps the chord chart for the default arrangement. `--transpose` accepts named
keys (C, F#, Bb) or relative semitone offsets (-2, +3). When a song has
multiple arrangements, prints available arrangement names at the end.

```sh
elvanto songs chart 84937df8-e993-11e2-b739-a20c5589acc5
elvanto songs chart 84937df8-e993-11e2-b739-a20c5589acc5 --transpose G
elvanto songs chart 84937df8-e993-11e2-b739-a20c5589acc5 --arrangement "Acoustic"
```

### `songs lyrics`

Prints lyrics for the default arrangement.

```sh
elvanto songs lyrics 84937df8-e993-11e2-b739-a20c5589acc5
elvanto songs lyrics 84937df8-e993-11e2-b739-a20c5589acc5 --arrangement "Acoustic"
```

## JSON Normalization

See [ADR 0001](adr/0001-normalized-json-output.md). Key rules:

- Flatten wrappers (no `songs.song[]`)
- `number` → `ccli_number`
- Booleans as `true`/`false`, not `0`/`1`
- Status as strings ("active", "archived")

## Error Handling

```
error: Elvanto returned code 250: No search parameters provided.
error: network: connection timed out
```

Exit codes: 0 (success), 1 (API error), 2 (usage/config).

## Security

- `ELVANTO_API_KEY` env var only, loaded from the shell or `.env`; no flag for passing a key
- Never print keys or auth headers
- Redact credentials in `--verbose` output

## References

- [Elvanto API: Getting Started](https://www.elvanto.com/api/getting-started/)
- [Elvanto API: Songs](https://www.elvanto.com/api/songs/)
- [Elvanto API: songs/getAll](https://www.elvanto.com/api/songs/getAll/)
- [Elvanto API: songs/getInfo](https://www.elvanto.com/api/songs/getInfo/)
- [Elvanto API: songs/categories/getAll](https://www.elvanto.com/api/songs/categories/getAll/)
- [Elvanto API: songs/arrangements/getAll](https://www.elvanto.com/api/songs/arrangements/getAll/)
- [Elvanto API: songs/arrangements/getInfo](https://www.elvanto.com/api/songs/arrangements/getInfo/)
