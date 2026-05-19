# `elvanto songs`

Worship song commands. The V1 surface is read-only — mutation commands are
planned but not yet implemented.

## Subcommands

| Command | Purpose |
| --- | --- |
| [`songs categories`](#songs-categories) | List song categories |
| [`songs list`](#songs-list) | List active songs (paginated through `songs/getAll`) |
| [`songs show`](#songs-show) | Show details of a single song |
| [`songs chart`](#songs-chart) | Print the chord chart for a song's default arrangement |
| [`songs lyrics`](#songs-lyrics) | Print the lyrics for a song's default arrangement |

---

## `songs categories`

```text
elvanto songs categories [--id short|long|hidden] [--json]
```

Lists every song category in the account:

```
45d9abe5 | Contemporary
63b140bf | Modern Worship
8a1c2b3d | Hymns
```

Columns: `id | name`. Use the id with `songs list --category` to filter.

### Flags

| Flag | Description |
| --- | --- |
| `--id <MODE>` | `short` (default), `long`, or `hidden`. See [IDs](../concepts/ids.md). |
| `--json` | Emit normalized JSON. |

## `songs list`

```text
elvanto songs list [--category ID]... [--used-within DUR] [--not-used-within DUR]
                   [--last-used] [--album] [--ccli] [--id short|long|hidden] [--json]
```

Lists active songs (paginated through `songs/getAll`). JSON output includes
non-active songs.

Default text output:

```
45d9abe5 | Holy Forever        | Chris Tomlin
63b140bf | King of Kings       | Hillsong Worship
```

Columns: `id | title | artist`. With `--last-used` the most recent service
date is appended; with `--album` and `--ccli` those columns are appended.

### Flags

| Flag | Description |
| --- | --- |
| `--category <ID>` | Keep songs assigned to this category. Repeat for OR. |
| `--used-within <DURATION>` | Keep songs used in a service within this duration. |
| `--not-used-within <DURATION>` | Exclude songs used in a service within this duration. |
| `--last-used` | Include the most recent service date column; also sorts most-recent-first. |
| `--album` | Include the album column. |
| `--ccli` | Include the CCLI number column. |
| `--full-id` | Print full song UUIDs. |
| `--json` | Emit normalized JSON; includes non-active songs. |

### Duration syntax

`--used-within` and `--not-used-within` accept short durations:

| Suffix | Meaning |
| --- | --- |
| `d` | days |
| `w` | weeks |
| `m` | months (30-day approximation) |
| `y` | years |

Examples: `14d`, `2w`, `6m`, `1y`.

### Examples

```sh
# Contemporary + Modern, used in the last 6 months, not in the last 14 days,
# sorted most-recently-used first
elvanto songs list \
  --category 45d9abe5 --category 63b140bf \
  --used-within 6m --not-used-within 14d \
  --last-used

# Cold-storage candidates: not used in 2+ years
elvanto songs list --not-used-within 2y
```

## `songs show`

```text
elvanto songs show <ID> [--full] [--files] [--json]
```

Prints a single song. The default text view is compact; `--full` expands
metadata fields (excluding lyrics and chord chart, which live behind
[`songs lyrics`](#songs-lyrics) and [`songs chart`](#songs-chart)).

### Flags

| Flag | Description |
| --- | --- |
| `--full` | Expand all metadata fields in text output. |
| `--files` | Include attachment data. Only meaningful with `--json`. |
| `--json` | Emit the full normalized song object as JSON. |

## `songs chart`

```text
elvanto songs chart <SONG_ID> [--arrangement ID] [--transpose KEY|OFFSET]
```

Streams the chord chart for the song's default arrangement (or the first
arrangement if none is marked default) to stdout. Pipe into `less`, save with
`>`, or feed into a printer.

### Flags

| Flag | Description |
| --- | --- |
| `--arrangement <ID>` | Use a specific arrangement by id (full UUID or short first-block) instead of the default. Use `elvanto songs show <song>` to look up ids. |
| `--transpose <KEY\|OFFSET>` | Transpose to a named key (`C`, `F#`, `Bb`) or a relative semitone offset (`-2`, `+3`). |

### Examples

```sh
# Save the default arrangement as a text file
elvanto songs chart 45d9abe5 > holy-forever.chart

# Transpose up a whole step
elvanto songs chart 45d9abe5 --transpose +2

# A specific arrangement (short id), in F#
elvanto songs chart 45d9abe5 --arrangement 7c81abe5 --transpose 'F#'
```

## `songs lyrics`

```text
elvanto songs lyrics <SONG_ID> [--arrangement ID]
```

Streams the lyrics for the song's default arrangement (or `--arrangement`) to
stdout. No transposition, no chords — text only. `--arrangement` takes an
arrangement id (full or short), same as `songs chart`.
