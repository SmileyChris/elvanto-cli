# Elvanto CLI

A Rust CLI for the Elvanto API, focused on scriptable access to worship song data.

## Language

**Song**:
A worship song with musical structure: arrangements, keys, lyrics, chord charts, CCLI number.
_Avoid_: Track, hymn, piece

**Item**:
A non-musical service element (e.g., "Welcome", "Announcements", "Offering"). Lives in the same Elvanto API surface but is out of scope for song commands.
_Avoid_: Song (when the thing has no music)

**Arrangement**:
A version of a song with a name, optional sequence, duration, BPM, lyrics, and chord chart. Every song has at least one arrangement. Multiple arrangements represent different styles or settings of the same song.
_Avoid_: Version, style

**Key**:
A playable musical key for an arrangement. Has a starting key and optional ending key.
_Avoid_: Transposition, capo (these are operations on a key, not the key itself)

**Category**:
A grouping label for songs (e.g., "Worship", "Hymns"). Used for filtering in Elvanto.
_Avoid_: Tag, genre, type

**CCLI number**:
The Copyright Licensing International identifier for a song. Stored as `number` in the Elvanto API.
_Avoid_: Song number, license number

## V1 commands (read-only)

```
elvanto auth check
elvanto songs categories           [--json]
elvanto songs list                  [--json] [--album] [--ccli]
elvanto songs show <id>             [--json] [--full] [--files]
elvanto songs chart <id>            [--transpose KEY|OFFSET] [--arrangement NAME]
elvanto songs lyrics <id>           [--arrangement NAME]
```

Global flags: `--verbose`

Auth: `ELVANTO_API_KEY` env var (required). No `--api-key` flag, no config file, no `--base-url` in V1.

### Output defaults

- `songs list` defaults to text: `id | title | artist`. `--album` and `--ccli` add those columns
- `songs list` filters to active songs by default in text mode, returns all in `--json`
- `songs list` auto-fetches all pages by default
- `songs chart` dumps chord chart text as-is
- `songs chart` and `songs lyrics` are human-only; no `--json` — use `songs show --json` for programmatic
- `--files` on `songs show --json` includes attachment data; deferred from human output
- Transposition accepts named keys (C, F#, Bb) or relative semitone offsets (-2, +3)
- When a song has multiple arrangements, `songs chart` and `songs lyrics` pick "Default" if it exists, otherwise the first, and list available arrangements as a hint at the end

### `songs show` output tiers

- **Default (curated):** title, artist, CCLI number, status, first line of lyrics, arrangement/key summary (one line each)
- **`--full`:** all API fields except lyrics and chord chart (those belong to `songs chart` / `songs lyrics`). Adds: categories, locations, notes, sequence, BPM, duration, learn/allow_downloads flags. Partially replaces default (e.g. full lyrics text replaces the first-line preview).
- **`--json`:** normalized JSON of the full song object

### JSON normalization

- Flatten wrapper objects (no `songs.song[]`, just a clean array)
- Rename `number` → `ccli_number`
- Booleans as `true`/`false`, not `0`/`1`
- Keep clear names as-is (`id`, `title`, `artist`, `status`, `name`)
- Status values normalized to strings ("active" not `1`)

### `auth check`

- Shows redacted API key (first 4 + last 4 chars)
- Exits 0 on valid, non-zero on invalid
- No account name available via API key (Elvanto has no account info endpoint); OAuth users get username via `people/currentUser`

### Error output

- Show Elvanto error code + message: `error: Elvanto returned code 250: No search parameters provided.`
- Network errors are distinguishable: `error: network: connection timed out`
- Exit codes: 0 (success), 1 (API error), 2 (usage/config)

## Relationships

- A **Song** has one or more **Arrangements**
- An **Arrangement** has zero or more **Keys**
- A **Song** may belong to zero or more **Categories**

## Example dialogue

> **Dev:** "How do I find all the Keys for a Song?"
> **Domain expert:** "You don't need a separate command — `songs show <id>` gives you the song with every arrangement and its keys. If you want just the keys for scripting, use `--json` and pipe through `jq`."
>
> **Dev:** "Is a service element like 'Welcome' a Song?"
> **Domain expert:** "No — that's an Item. The song commands only deal with actual worship songs."

**Chord chart**:
A text representation of a song's chord progression, retrieved via `songs chart <id>`. Supports transposition via `--transpose`.

**Lyrics**:
The full text of a song's verses, choruses, and bridges, retrieved via `songs lyrics <id>`.

## Flagged ambiguities

- "number" in the Elvanto API is the CCLI number, not an internal Elvanto identifier. Resolved: use **CCLI number** in output, keep `number` in API structs.
