# Normalize JSON output instead of mirroring Elvanto's raw response shape

`--json` output flattens wrapper objects, renames confusing fields, and
normalizes types instead of passing through the raw Elvanto response. The API
client module keeps raw types internally; conversion happens at the output
boundary.

## Why

Elvanto's JSON responses carry structural noise: responses wrap results in
`songs.song[]`, `arrangements.arrangement[]`, etc. Field names like `number`
(actually the CCLI number) and numeric booleans (`1`/`0`) require every consumer
to decode them. Normalizing once in the CLI lets scripts, jq pipelines, and
humans read the output without Elvanto-specific decoding.

## Considered Options

**Pass through raw API responses.** Simpler to implement, but every consumer
must unwrap `songs.song[]` and decode `1`/`0` booleans. We'd end up writing the
same jq transforms in every script that uses this tool.

**Two output modes (raw + normalized).** More complexity in the CLI for a mode
we'd rarely use. If someone needs raw responses they can call the Elvanto API
directly.

Picked normalization as the only path because the CLI's value is in making
Elvanto data usable without ceremony.
