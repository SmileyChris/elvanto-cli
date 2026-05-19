# IDs and short IDs

Elvanto identifies everything (songs, services, people, departments,
categories, positions) with full v1 UUIDs:

```
02b06b47-c275-11e6-aad3-0219ad55c99b
```

These are stable, unique, and impossible to remember. `elvanto-cli` shortens
them for terminal output and accepts the short form on the command line.

## What "short id" means

A short id is the **first dash-separated block** of the UUID:

| Full | Short |
| --- | --- |
| `02b06b47-c275-11e6-aad3-0219ad55c99b` | `02b06b47` |
| `bf40484c-1111-2222-3333-444444444444` | `bf40484c` |

The first block is the time-low portion of a v1 UUID. Inside any one Elvanto
account, the first 8 hex digits are effectively unique across all entity types
you'll list together (a few hundred to a few thousand items per list).

## Where short ids appear

Every list command renders the short id by default:

```
$ elvanto songs list
45d9abe5 | Holy Forever | ...
63b140bf | King of Kings | ...

$ elvanto people departments
04050825 | Instruments | sub_department | Music Team
80be1d73 |   Setup & Cleanup | position | Communion
```

Every list command takes `--id short|long|hidden`:

- `--id short` (default) — first dash-separated block, as above.
- `--id long` — full UUIDs.
- `--id hidden` — drop the id column entirely.

JSON output (`--json`) always uses full UUIDs regardless of this flag — that's
the stable programmatic contract.

## Where short ids are accepted

Any flag or positional that takes an id accepts either a full UUID or a short
form. Matching is done by `id_matches(full, requested)`, which returns true if
either the full id is an exact match or the short form is.

Concretely:

```sh
# Both work — same song
elvanto songs show 45d9abe5
elvanto songs show 45d9abe5-1234-5678-9abc-def012345678

# --category, --in, etc. all use the same matcher
elvanto songs list --category 45d9abe5
elvanto people list --in bf40484c
elvanto services people 02b06b47 --in 03fa8320
```

## Caveats

- **Short ids are stable but not globally unique.** Two entities of different
  types could in principle share an 8-hex-digit prefix; `elvanto` matches
  within the result set you're filtering, so collisions in practice are very
  rare but not impossible.
- If you store ids in scripts or `.env` files, **use the short form for
  ergonomics, the full form for safety.** The CLI accepts both.
