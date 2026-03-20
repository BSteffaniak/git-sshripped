# git_sshripped_filter

Git clean/smudge/diff filter operations for git-sshripped.

## Overview

This crate bridges the encryption layer to Git's filter mechanism. It
implements the three filter operations that run transparently during normal Git
workflows:

- **clean** -- encrypts plaintext when staging (`git add`). Errors if the
  repository is locked and the file is unencrypted.
- **smudge** -- decrypts ciphertext when checking out. Passes ciphertext
  through unchanged when the repository is locked.
- **diff** -- decrypts ciphertext for textconv display. Errors if the
  repository is locked.

Each function accepts an optional `repo_key`; a `None` key indicates the
repository is locked.

## Key Functions

- `clean(algorithm, repo_key, path, content)` -- encrypt for staging.
- `smudge(repo_key, path, content)` -- decrypt for checkout.
- `diff(repo_key, path, content)` -- decrypt for diff display.

## Usage

Part of the [git-sshripped](https://github.com/BSteffaniak/git-sshripped)
workspace. This crate is not intended for standalone use.
