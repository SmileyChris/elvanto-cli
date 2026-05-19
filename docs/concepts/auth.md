# Authentication

`elvanto-cli` uses Elvanto's API-key authentication (HTTP Basic Auth, key as
username, dummy password). It does **not** implement OAuth at the moment.

## Where the key comes from

For every command that needs a key, `elvanto` resolves it in this order:

1. **The `ELVANTO_API_KEY` environment variable.** If set to a non-empty value,
   it wins outright. This is what `.env` files populate.
2. **The OS keyring**, under service `elvanto-cli`. Populated and managed by
   `elvanto auth login` and `elvanto auth clear`.
3. **Nothing found** → the command fails with:

   ```text
   error: no API key found; set ELVANTO_API_KEY or run `elvanto auth login`
   ```

`elvanto auth login`, `elvanto auth clear`, and `elvanto auth status` bypass
this resolver — they always operate on the keyring directly. `auth status`
reports which source would be used and verifies the resolved key against the
live API.

## Picking a method

| Use case | Recommended |
| --- | --- |
| Interactive workstation | OS keyring (`elvanto auth login`) |
| Project-specific overrides | Repo-local `.env` file (gitignored) |
| CI / containers | `ELVANTO_API_KEY` environment variable |
| One-off invocation | Inline: `ELVANTO_API_KEY=... elvanto songs list` |

## `.env` files

`elvanto-cli` looks for a `.env` file in the current working directory or any
parent directory (via [`dotenvy`](https://crates.io/crates/dotenvy)). Variables
already set in the process environment are **not** overridden — so explicit
`export ELVANTO_API_KEY=...` wins over `.env`.

A `.env` for a typical worship-team workstation might look like:

```dotenv
ELVANTO_API_KEY=your-key-here

ELVANTO_SONGS_LIST="--category 45d9abe5 --used-within 6m --last-used"
ELVANTO_SERVICES_PEOPLE="--hide-unfilled --email"
```

`.env` is in the project's `.gitignore` — keep your team's keys out of source
control.

## OS keyring backend

The keyring is provided by [`keyring` v3](https://crates.io/crates/keyring)
with native backends:

- **Linux:** D-Bus Secret Service (gnome-keyring, KWallet).
- **macOS:** Keychain.
- **Windows:** Credential Manager.

If the backend is unavailable (headless Linux, no D-Bus), `auth login` and
`auth status` surface the underlying error so you can fall back to
`ELVANTO_API_KEY`.

## Verifying it works

```sh
elvanto auth status   # shows source + API verification
```

`auth status` exits non-zero on failure, so it fits cleanly into `set -e`
scripts.
