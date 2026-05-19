# Auto-injected flags

Most teams run a small set of commands with the same flags every day. Instead
of building shell aliases, `elvanto-cli` lets you put those default flags in
the environment, scoped per subcommand.

## How it works

For each subcommand path, `elvanto` looks up an environment variable named
`ELVANTO_<PATH_JOINED_BY_UNDERSCORE>` (uppercased). The value is shell-tokenised
(via [`shlex`](https://crates.io/crates/shlex)) and spliced into argv directly
after the subcommand path.

| Subcommand | Environment variable |
| --- | --- |
| `elvanto songs list` | `ELVANTO_SONGS_LIST` |
| `elvanto songs show` | `ELVANTO_SONGS_SHOW` |
| `elvanto services list` | `ELVANTO_SERVICES_LIST` |
| `elvanto services people` | `ELVANTO_SERVICES_PEOPLE` |
| `elvanto people list` | `ELVANTO_PEOPLE_LIST` |
| `elvanto people departments` | `ELVANTO_PEOPLE_DEPARTMENTS` |

(Anything two levels deep follows the same `ELVANTO_GROUP_LEAF` pattern.)

## Opt-out rules

Injection is **disabled** in any of the following cases:

1. The user passed the global `--no-env` flag.
2. The user supplied **any** `-`-prefixed flag after the subcommand path. The
   first manual flag fully overrides the env-injected defaults — they are not
   merged.
3. The variable is unset, empty, or fails to tokenise (unbalanced quotes, etc.).

This gives a "defaults you can step out of" model: the env vars apply when
you're running the command bare, but adding any flag puts you back in
fully-explicit territory.

## You'll see a stderr note

When an env var is set, the CLI emits a stderr note after the command finishes
so you can spot when defaults are in effect — or have just been silently
suppressed:

```text
$ elvanto songs list
... output ...
note: applied ELVANTO_SONGS_LIST defaults (use --no-env to disable):
      --category 45d9abe5 --used-within 6m --last-used
```

```text
$ elvanto songs list --json
... output ...
note: ELVANTO_SONGS_LIST defaults suppressed by manual flag (use --no-env to silence this note):
      would have applied: --category 45d9abe5 --used-within 6m --last-used
```

Pass `--no-env` to silence the note entirely (and unconditionally disable
injection):

```text
$ elvanto --no-env songs list --json
... output ...
```

The note goes to stderr — it never pollutes stdout, so it's safe to pipe into
`jq`, `cut`, etc.

## Examples

Set a default category and recency window for songs:

```dotenv
ELVANTO_SONGS_LIST="--category 45d9abe5 --used-within 6m --last-used"
```

```sh
# Auto-injects flags above:
elvanto songs list

# Manual flag → injection skipped, runs as written:
elvanto songs list --json

# Bypass without changing the command:
elvanto --no-env songs list
```

Set a default filter for `services people` (worship team only, with email):

```dotenv
ELVANTO_SERVICES_PEOPLE="--hide-unfilled --in bf40484c --in 03fa8320 --email"
```

```sh
elvanto services people 02b06b47
# becomes:
# elvanto services people --hide-unfilled --in bf40484c \
#                          --in 03fa8320 --email 02b06b47
```

Positional arguments after the path (like the service id above) are preserved.

## Limits

- Only two levels of subcommand path are inspected.
- Single-level commands (`elvanto <name>`) are supported but require `<name>`
  to be a real subcommand of `elvanto`.
- The env var is parsed as a single shell-quoted string — newlines or comments
  are not supported.
