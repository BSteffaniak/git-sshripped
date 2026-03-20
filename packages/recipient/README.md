# git_sshripped_recipient

Recipient keyring management for git-sshripped.

## Overview

This crate manages the set of SSH public-key recipients authorized to access a
repository. Recipients are stored as TOML files under
`.git-sshripped/recipients/`, keyed by their SHA256 fingerprint.

Key responsibilities:

- **Recipient CRUD** -- add, list, and remove recipients from the keyring.
- **GitHub key fetching** -- fetch SSH public keys for individual GitHub users
  and GitHub team members, using either the `gh` CLI or the REST API with
  ETag-based conditional requests and rate-limit awareness.
- **Fingerprinting** -- compute URL-safe base64 SHA256 fingerprints for SSH
  public keys.
- **Key wrapping** -- wrap and unwrap the repository data key using the `age`
  library's SSH encryption. Each recipient gets an individually encrypted copy
  of the repo key under `.git-sshripped/keys/`.

## Key Functions

- `list_recipients()` / `add_recipient_from_public_key()` /
  `remove_recipient_by_fingerprint()`
- `fetch_github_user_keys()` / `fetch_github_team_members()`
- `wrap_repo_key_for_recipient()` / `wrap_repo_key_for_all_recipients()`
- `fingerprint_for_public_key_line()`

## Usage

Part of the [git-sshripped](https://github.com/BSteffaniak/git-sshripped)
workspace. This crate is not intended for standalone use.
