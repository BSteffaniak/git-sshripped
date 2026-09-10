# Security Model

This project is currently pre-1.0 and should be treated as security-sensitive software under active hardening.

## Threat model

- Protect protected-path file contents at rest in Git objects and on remotes.
- Assume attacker can read repository history and encrypted blobs.
- Assume attacker does not control maintainer/developer endpoints with unlocked keys.

## Non-goals

- Defense against local host compromise after unlock.
- Forward secrecy for already-encrypted history.
- Metadata hiding for protected paths and file sizes.

## Current cryptographic properties

- Repository files are encrypted with a repository data key.
- Repository data key is wrapped per recipient SSH public key.
- New ciphertext is movable by default and uses fixed associated data.
- Path binding is optional; strict paths use repository-relative path bytes as
  associated data. See [FORMAT.md](docs/FORMAT.md).
- The clean filter rejects protected plaintext while locked, provided the Git
  filter/attributes are correctly configured. This is not a guarantee against
  bypassing filters or committing secrets at unprotected paths.

## Deterministic leakage

Deterministic encryption is used for Git filter stability. This leaks:

- Equality of same plaintext at same path/AD context.
- Approximate plaintext length.

## Operational requirements

- Keep at least two valid recipients to avoid lockout.
- Run `git-sshripped doctor` and `git-sshripped verify --strict` in CI.
- Rotate recipients/keys when access policy changes.
- Prefer `ssh-ed25519` recipients; use `ssh-rsa` only when compatibility requires it.

## Reporting security issues

Report vulnerabilities privately to [bradensteffaniak@gmail.com](mailto:bradensteffaniak@gmail.com)
before opening public issues. Include the version, Git version, platform, and a
synthetic reproduction. Do not send live private keys or repository secrets.
There is no promised response time or independent-audit claim.
