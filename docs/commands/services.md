# `elvanto services`

Services (calendar event) commands.

## Subcommands

| Command | Purpose |
| --- | --- |
| [`services list`](#services-list) | List services in a date range (defaults to the last 6 months) |
| [`services people`](#services-people) | Show volunteers assigned to a specific service |
| [`services song-usage`](#services-song-usage) | Analyse how often each song has been used |

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
| `--id <MODE>` | `short` (default), `long`, or `hidden`. See [Lookups](../concepts/lookups.md). |
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
| `--in <ID\|NAME\|PATH>` | Keep rows whose department, sub-department, **or** position matches. Accepts the same id/name/path forms as [`people list`](people.md#people-list); the resolver only sees nodes on this service. |
| `--email` | Include each person's primary email as an extra column. |
| `--id <MODE>` | `short` (default), `long`, or `hidden` for the person-id column. See [Lookups](../concepts/lookups.md). |
| `--json` | Emit normalized JSON. Includes ids at every level. |

### Examples

```sh
# Full team for one service
elvanto services people 02b06b47

# Worship team only, hide gaps, include emails — by name
elvanto services people 02b06b47 \
  --in "Service Leaders" --in Vocals \
  --hide-unfilled --email

# Same thing by id (better for .env defaults)
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

## `services song-usage`

```text
elvanto services song-usage [--from YYYY-MM-DD] [--to YYYY-MM-DD] [--max-uses N]
```

Counts how many times each song has been used across services, and lists those
sung N or fewer times (default: 2). Includes who led each time. Useful for
finding songs that could use more rotation.

Text output (no `--json` mode):

```
Songs sung ≤ 2 times in the last 12 months:

"Lord I Need You" by Christy Nockels, ... — sung once
  2025-11-22 — Led by Lara Coates

"Mighty To Save" by Ben Fielding, Reuben Morgan — sung once
  2026-05-23 — Led by Chris Beaven

"O Come To The Altar" by Chris Brown, ... — sung twice
  2025-11-29 — Led by Chris Beaven
  2026-02-21 — Led by Chris Beaven
```

### Flags

| Flag | Description |
| --- | --- |
| `--from <YYYY-MM-DD>` | Inclusive start date. Defaults to 12 months before `--to`. |
| `--to <YYYY-MM-DD>` | Inclusive end date. Defaults to today. |
| `--max-uses <N>` | Only show songs used ≤ N times (default: 2). |
| `--json` | Emit all song usage as JSON (ignores `--max-uses` / `--one-leader`). |

### Examples

```sh
# Songs used 1-2 times in the last 12 months
elvanto services song-usage

# Full usage history as JSON (for scripts)
elvanto services song-usage --json

# Songs used ≤ 3 times since the start of 2026
elvanto services song-usage --from 2026-01-01 --max-uses 3
```
