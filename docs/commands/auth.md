# `elvanto auth`

Manage and verify API-key credentials. See [Authentication](../concepts/auth.md)
for the full resolution rules.

## Subcommands

| Command | Purpose |
| --- | --- |
| [`auth login`](#auth-login) | Store an API key in the OS keyring |
| [`auth clear`](#auth-clear) | Remove the keyring-stored key |
| [`auth status`](#auth-status) | Report source (env / keyring / none) + verify against Elvanto |

---

## `auth login`

```text
elvanto auth login [--stdin]
```

Prompts interactively for an API key (input is hidden via
[`rpassword`](https://crates.io/crates/rpassword)) and stores it in the OS
keyring. The stored key is then used as the fallback after `ELVANTO_API_KEY`.

| Flag | Description |
| --- | --- |
| `--stdin` | Read the key from stdin (one line) instead of prompting. Useful in scripts. |

```sh
# Interactive
elvanto auth login

# Scripted
printf '%s\n' "$ELVANTO_API_KEY" | elvanto auth login --stdin
```

A successful login prints `stored API key in keyring` to stderr.

## `auth clear`

```text
elvanto auth clear
```

Deletes the keyring entry. Prints `removed API key from keyring` if a key was
present, or `no API key stored in keyring` if nothing was stored. Does not
touch the environment.

## `auth status`

```text
elvanto auth status
```

Reports which source `elvanto` would use for the API key (`env (ELVANTO_API_KEY)`,
`keyring`, or `none`), prints the redacted key, and performs a live
verification by hitting `songs/categories/getAll`.

Typical output:

```text
source: env (ELVANTO_API_KEY)
key:    abcd…wxyz
status: ok
```

or, with a bad key:

```text
source: env (ELVANTO_API_KEY)
key:    wron…rong
status: invalid — Elvanto returned code 121: Invalid API Key.
```

or, with nothing configured:

```text
source: none
status: no API key — run `elvanto auth login` or set ELVANTO_API_KEY
```

Exits non-zero if no key is available or if Elvanto rejects the key — so it
fits cleanly into `set -e` CI scripts.
