# Encrypted File Format

## Header

- Magic: `GSC1` (4 bytes)
- Version: `1` (1 byte)
- Algorithm ID (1 byte)
  - `1` for `AesSivV1` legacy path-bound AES-SIV
  - `2` for `AesSivMovableV1` movable AES-SIV

Header length is 6 bytes total.

## Payload

- Ciphertext produced by deterministic AES-SIV encryption backend.
- Algorithm ID `1` uses the repository-relative path bytes as associated data.
  This is the legacy format and remains readable for compatibility.
- Algorithm ID `2` uses fixed associated data (`git-sshripped:aes-siv:movable:v1`)
  so encrypted files can move paths without re-encryption. This is the default
  for newly encrypted files.

## Parsing rules

- Magic mismatch -> not encrypted in this format.
- Unknown version/algorithm -> parse failure.
- Decrypt failure -> integrity/authentication failure.

## Path binding

Movable encryption is the default because git-sshripped's primary guarantee is
that protected file contents are encrypted in Git history. Path-bound encryption
is available as an opt-in `.gitattributes` policy for files that should also
reject ciphertext moved from another path:

```gitattributes
prod-secrets/** filter=git-sshripped diff=git-sshripped git-sshripped-path-binding=strict
```

Use `git-sshripped init --path-binding strict --pattern "prod-secrets/**"` to
install that attribute for initial patterns. Use
`git-sshripped policy set --default-path-binding <none|strict>` to change the
repository manifest default used by patterns without an explicit attribute.

## Compatibility

- New algorithms must add a new algorithm ID.
- Version upgrades must preserve backward parsing where feasible.
