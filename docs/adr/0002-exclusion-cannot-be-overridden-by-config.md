# Exclusion cannot be overridden by config or ignore files

Files matching the built-in secret-file list (`.env*`, `*.pem`, `*.key`, `id_rsa*`, `*.p12`, `credentials.json`, …) are never read, and no ignore file, `--include` filter or `pithpack.toml` can remove an entry from that list; config may only add to it. The single escape hatch is naming the exact path on the command line together with `--allow-secret-file`. Likewise Redaction can only be disabled with the `--no-redact` flag, never from config. A Bundle is often pasted straight into a third-party API, so a leak is irreversible; making the override require a deliberate, per-invocation act means a committed `pithpack.toml` or a negated `.gitignore` pattern can never silently start shipping secrets.

## Considered Options

- **Treat the list as ordinary default ignores**: rejected; a `!.env` line in `.gitignore` or a broad `--include` would undo it without anyone noticing.
- **Allow a config-level `allow_secret_files = true`**: rejected for the same reason: config is shared and persistent, the risk is per-run.
