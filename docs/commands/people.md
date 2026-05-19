# `elvanto people`

Directory commands: list people, explore the department/sub-department/position
tree, and filter by id at any level.

## Subcommands

| Command | Purpose |
| --- | --- |
| [`people list`](#people-list) | List active people, optionally filtered by org-tree id |
| [`people departments`](#people-departments) | Show the full department / sub-department / position tree |

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
| `--in <ID>` | Keep people whose department, sub-department, **or** position id matches the given id. Repeat for OR across multiple ids. Accepts full UUIDs or short ids. |
| `--id <MODE>` | `short` (default), `long`, or `hidden`. See [IDs](../concepts/ids.md). |
| `--json` | Emit normalized JSON instead of text. JSON output ignores `--id` and always uses full UUIDs. |

### Filtering by org tree

The `--in` flag matches **by id at every level of the tree** (department,
sub-department, or position). There is intentionally only one filter flag —
no separate `--sub-department` or `--position`. Real-world Elvanto
departments often share names ("Volunteer", "Set-Up", "Leader") at different
levels, so id-based filtering is the only way to be unambiguous.

```sh
# By top-level department id
elvanto people list --in d-1-...

# By sub-department id (e.g. "Vocals")
elvanto people list --in 03fa8320

# By position id (e.g. "Worship Leader")
elvanto people list --in <position-id>

# Multiple OR-matched ids
elvanto people list --in bf40484c --in 03fa8320
```

Use [`people departments`](#people-departments) to look up the ids.

## `people departments`

```text
elvanto people departments [--id short|long|hidden] [--json]
```

Prints the **full** department tree — every top-level department, its
sub-departments, and every position under those — as a flat list in
depth-first order, with indentation in the `name` column for visual hierarchy:

```
d-1     | Music Team       | department     | -
sd-1    |   Vocals         | sub_department | Music Team
p-wl    |     Worship Leader | position     | Vocals
p-bv    |     BV           | position       | Vocals
sd-2    |   Instruments    | sub_department | Music Team
d-2     | Welcome Team     | department     | -
```

Columns: `id | name | kind | parent`. `kind` is one of `department`,
`sub_department`, `position`. The `parent` for top-level departments is `-`.

### Flags

| Flag | Description |
| --- | --- |
| `--id <MODE>` | `short` (default), `long`, or `hidden`. See [IDs](../concepts/ids.md). |
| `--json` | Emit normalized JSON. |

### Notes

- Departments are only discovered by walking `people/getAll` — the API does
  not expose a dedicated list-all-departments endpoint. So this command
  always paginates through all people.
- Output is deduplicated and sorted alphabetically within each level.
