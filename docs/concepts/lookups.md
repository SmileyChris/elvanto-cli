# Lookups: IDs, names, and paths

Every flag that takes an entity reference — `--in`, `--category`,
`--arrangement`, the `<ID>` positional on `songs show` / `songs chart` /
`services people` — accepts the same three input forms:

| Form | Example | Notes |
| --- | --- | --- |
| **Full UUID** | `02b06b47-c275-11e6-aad3-0219ad55c99b` | The id Elvanto returns |
| **Short id** | `02b06b47` | First dash-separated block of the UUID |
| **Name or path** | `Vocals`, `Music Team/Vocals` | Resolved against the relevant tree at runtime |

ID-shaped input is always treated as an id (and fails hard if no node has
that id — no name-search fallback). Anything else is treated as a name or
path.

## Short IDs

A short id is the **first dash-separated block** of a v1 UUID:

| Full | Short |
| --- | --- |
| `02b06b47-c275-11e6-aad3-0219ad55c99b` | `02b06b47` |
| `bf40484c-1111-2222-3333-444444444444` | `bf40484c` |

Inside one Elvanto account, the first 8 hex digits are effectively unique
across the entity types you'll list together. Two entities of different
types *could* in principle share a prefix, but in practice it's rare.

List commands render short ids by default:

```
$ elvanto songs list
45d9abe5 | Holy Forever | ...
63b140bf | King of Kings | ...

$ elvanto people org
d7341d20 | Cafe Team
d73545dc |   Cafe
d7362b3d |     Barista
```

Every list command takes `--id short|long|hidden`:

- `--id short` (default) — first dash-separated block.
- `--id long` — full UUIDs.
- `--id hidden` — drop the id column entirely.

JSON output (`--json`) always uses full UUIDs regardless — that's the stable
programmatic contract.

## Name and path lookup

When you pass something that doesn't look like an id, the CLI resolves it
against the org tree (or, for `--category`, the category list):

- `--in "Vocals"` — match any node whose last path segment is `Vocals`.
- `--in "Music Team/Vocals"` — match a node whose path ends with
  `[Music Team, Vocals]`.
- `--in "Music Team"` — match the dept itself. Because of how the matcher
  works, that also catches everyone in any sub-dept or position under it.
  Parent match = whole subtree, for free.

Matching is case-insensitive, whole-segment, and uses `/` as the path
separator.

### Unique-prefix fallback

If no node matches the query exactly, the resolver retries with the last
query segment treated as a **prefix**. If exactly one node matches, that
node wins:

```sh
# Category is named "Contemporary (0-5 Years Old)" — only one category starts
# with "Contemporary", so this resolves to it.
elvanto songs list --category Contemporary

# Works in path queries too — only the last segment is prefix-matched.
elvanto people list --in "Music Team/Voc"   # → "Music Team / Vocals"
```

Exact match always wins over prefix. If two nodes both start with the
prefix, you'll get the same disambiguation table you'd get for any other
ambiguous query.

## Ambiguity is a feature

When a name matches more than one node, the resolver fails with a table —
which doubles as a `people org` invocation pre-filtered to your
query:

```
$ elvanto people list --in "Leader"
error: "Leader" matches 5 nodes — disambiguate with id or full path:
  bf40484c  position    Service Teams / Service Leaders / Leader
  03fa8320  position    Music Team / Vocals / Leader
  04050825  position    Music Team / Instruments / Leader
  0f91c170  position    Production / Tech / Leader
  ee882e1d  position    Welcome Team / Hosts / Leader
```

Pick the id or the path you want.

## Typos get fuzzy suggestions

Misses include a top-3 `Did you mean?` list when something close enough
(Jaro-Winkler ≥ 0.7) exists:

```
$ elvanto people list --in "voclas"
error: no match for "voclas". Did you mean:
  03fa8320  sub_department  Music Team / Vocals
```

IDs that look like UUIDs but don't match anything fail without suggestions —
the CLI assumes you meant to paste a real id and doesn't try to second-guess.

## Where the tree comes from

| Command | Tree source | Extra API call? |
| --- | --- | --- |
| `people list --in` | Walks `person.departments` from the fetched people list | No |
| `services people --in` | Walks the volunteer rows for the requested service | No |
| `songs list --category` | Fetches `songs/categories/getAll` once when `--category` is set | Yes (small) |

`services people` only sees the org-tree nodes that actually appear on the
requested service. If you ask for `--in "Audio"` and no Audio volunteers
are scheduled, you'll get a `Did you mean?` suggesting whatever is on the
service — not the full org chart.

## When to use which form

| Use case | Recommended |
| --- | --- |
| `.env` defaults, CI scripts | Short id (stable, no name churn) |
| Ad-hoc interactive use | Name or path (faster to type, self-documenting) |
| Programmatic JSON pipelines | Full UUID (no ambiguity, ever) |
