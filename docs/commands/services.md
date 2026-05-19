# `elvanto services`

Services (calendar event) commands.

## Subcommands

| Command | Purpose |
| --- | --- |
| [`services list`](#services-list) | List services in a date range (defaults to the last 6 months) |
| [`services people`](#services-people) | Show volunteers assigned to a specific service |

---

## `services list`

```text
elvanto services list [--from YYYY-MM-DD] [--to YYYY-MM-DD] [--id short|long|hidden] [--json]
```

Lists services in a date range. By default, the range is the **last 6 months**
ending today (local time).

Text output:

```
02b06b47 | 2026-05-17 09:30 | Sunday Service
4d5e6f7a | 2026-05-10 09:30 | Sunday Service
```

Columns: `id | date | name`. The date is formatted in the local time zone.

### Flags

| Flag | Description |
| --- | --- |
| `--from <YYYY-MM-DD>` | Inclusive start date. Defaults to 6 months before `--to`. |
| `--to <YYYY-MM-DD>` | Inclusive end date. Defaults to today (local time). |
| `--id <MODE>` | `short` (default), `long`, or `hidden`. See [IDs](../concepts/ids.md). |
| `--json` | Emit normalized JSON. |

### Examples

```sh
# Last 6 months (default)
elvanto services list

# A specific quarter
elvanto services list --from 2026-01-01 --to 2026-03-31

# Just the upcoming month
elvanto services list --from "$(date -I)" --to "$(date -I -d '+1 month')"
```

## `services people`

```text
elvanto services people <ID> [--hide-unfilled] [--in ID]...
                              [--email] [--id short|long|hidden] [--json]
```

Shows the people assigned to a specific service, broken down by department,
sub-department, and position. The `<ID>` is the service id (short or full)
from `services list`.

Text output (without `--email`):

```
1eb01e76 | Service Leaders | Service Leader  | Alice Brown    | confirmed
1eb01e76 | Vocals          | Worship Leader  | Alice Brown    | confirmed
deadbeef | Vocals          | BV              | Bob Carter     | confirmed
abcd1234 | Instruments     | Acoustic Guitar | Carol Davies   | confirmed
-        | Instruments     | Bass            | (unfilled)     | -
feedcafe | Production      | FOH             | Dave Edwards   | confirmed
```

Columns: `person_id | sub_department | position | name | status`. Unfilled
positions render the id column as `-` and `(unfilled)` in the name column —
pass `--hide-unfilled` to drop them, or `--id hidden` to drop the id column
entirely.

### Flags

| Flag | Description |
| --- | --- |
| `--hide-unfilled` | Omit positions that have no person assigned. |
| `--in <ID>` | Keep rows whose department, sub-department, **or** position id matches. Repeat for OR. Same id-only matching semantics as [`people list`](people.md#people-list). |
| `--email` | Include each person's primary email as an extra column. |
| `--id <MODE>` | `short` (default), `long`, or `hidden` for the person-id column. See [IDs](../concepts/ids.md). |
| `--json` | Emit normalized JSON. Includes ids at every level. |

### Examples

```sh
# Full team for one service
elvanto services people 02b06b47

# Worship team only, hide gaps, include emails
elvanto services people 02b06b47 \
  --in bf40484c --in 03fa8320 \
  --hide-unfilled --email
```

JSON output:

```json
[
  {
    "department": "Music Team",
    "department_id": "d-1-...",
    "sub_department": "Vocals",
    "sub_department_id": "sd-1-...",
    "position": "Worship Leader",
    "position_id": "p-wl-...",
    "person_id": "1eb01e76-...",
    "name": "Alice Brown",
    "email": "alice@example.com"
  }
]
```

Unfilled positions appear in JSON with `person_id` and `name` omitted (when
`--hide-unfilled` is not set).
