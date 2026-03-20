# git_sshripped_worktree_models

Data types for per-worktree lock/unlock state in git-sshripped.

## Overview

This crate defines the serializable types that represent the unlock session
persisted inside each Git worktree. It contains no I/O logic -- only the shared
vocabulary used by the worktree and CLI crates.

## Key Types

- `UnlockSession` -- the session object written when a repository is unlocked,
  containing the base64-encoded repo key, a label for the key source, and an
  optional `repo_key_id`.
- `RepositoryLockState` -- enum with variants `Locked` and `Unlocked`.

## Usage

Part of the [git-sshripped](https://github.com/BSteffaniak/git-sshripped)
workspace. This crate is not intended for standalone use.
