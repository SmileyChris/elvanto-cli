# `elvanto people`

Directory commands: list people, explore the department/sub-department/position
tree, and filter at any level by id, name, or path.

## Subcommands

| Command | Purpose |
| --- | --- |
| [`people list`](#people-list) | List active people, optionally filtered by org-tree id |
| [`people org`](#people-org) | Show the full department / sub-department / position tree |

---

## `people list`

```text
elvanto people list [--in ID]... [--id short|long|hidden] [--json]
```

By default, prints **active** people only as a table:

```
1eb01e76 | Alice Brown | alice@example.com
deadbeef | Bob Carter  | bob@example.com
```

JSON output (`--json`) includes archived people and per-person department
entries. Each entry has up to three levels populated:

```json
[
  {
    "id": "1eb01e76-...",
    "name": "Alice Brown",
    "email": "alice@example.com",
    "status": "active",
    "departments": [
      {
        "department": "Music Team",
        "department_id": "d-1-...",
        "sub_department": "Vocals",
        "sub_department_id": "sd-1-...",
        "position": "Worship Leader",
        "position_id": "p-wl-..."
      }
    ]
  }
]
```

### Flags

| Flag | Description |
| --- | --- |
| `--in <ID\|NAME\|PATH>` | Keep people whose department, sub-department, **or** position matches. Accepts a full UUID, short first-block, name (`Vocals`), or path (`Music Team/Vocals`). Repeat for OR. See [Lookups](../concepts/lookups.md). |
| `--id <MODE>` | `short` (default), `long`, or `hidden`. See [Lookups](../concepts/lookups.md). |
| `--json` | Emit normalized JSON instead of text. JSON output ignores `--id` and always uses full UUIDs. |

### Filtering by org tree

`--in` matches at every level of the org tree. Pick whichever form is most
ergonomic — id, name, or path:

```sh
# By short id (stable for scripts and .env files)
elvanto people list --in 03fa8320

# By name (case-insensitive; errors if ambiguous)
elvanto people list --in Vocals
elvanto people list --in "Worship Leader"

# By path — disambiguates same-name nodes
elvanto people list --in "Music Team/Vocals"
elvanto people list --in "Music Team/Vocals/Leader"

# Parent match includes the whole subtree
elvanto people list --in "Music Team"   # everyone in Music Team or any child

# Multiple OR-matched lookups
elvanto people list --in Vocals --in "Welcome Team"
```

Ambiguous matches (e.g. `--in Leader` when several positions are named
"Leader") fail with a table of candidates. Typos get top-3 `Did you mean?`
suggestions. See [Lookups](../concepts/lookups.md) for full semantics.

## `people org`

```text
elvanto people org [--id short|long|hidden] [--json]
```

Prints the full organisational tree — every top-level department, its
sub-departments, and every position under those — as a flat list in
depth-first order, with indentation in the name column conveying depth:

```
d7341d20 | Cafe Team
d73545dc |   Cafe
d7362b3d |     Barista
d7382c2a |     Counter
d738f481 |     Manager
…
d3d88873 | 7up
dcb35040 |   7up Sunday (School Years 7 - 9)
0f9cb778 |     Leader
```

Two columns: `id | name`. JSON output (`--json`) keeps the full schema with
`kind` (`department` / `sub_department` / `position`) and `parent` fields
for programmatic consumers.

### Flags

| Flag | Description |
| --- | --- |
| `--id <MODE>` | `short` (default), `long`, or `hidden`. See [Lookups](../concepts/lookups.md). |
| `--json` | Emit normalized JSON (includes `kind` and `parent`). |

### Notes

- The tree is only discovered by walking `people/getAll` — the API does
  not expose a dedicated list-all-departments endpoint. So this command
  always paginates through all people.
- Output is deduplicated and sorted alphabetically within each level.
