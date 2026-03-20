# git_sshripped_worktree

Per-worktree lock/unlock session management for git-sshripped.

## Overview

This crate manages the transient unlock state that lives inside each Git
worktree. When a repository is unlocked, the repo key and metadata are
persisted as a JSON file at `.git/git-sshripped/session/unlock.json` (with
`0600` permissions on Unix). Locking removes the file.

The crate also resolves the Git common directory and working-tree root via
`git rev-parse`, correctly handling linked worktrees.

## Key Functions

- `git_common_dir(cwd)` / `git_toplevel(cwd)` -- resolve Git directories.
- `session_file(common_dir)` -- returns the path to `unlock.json`.
- `write_unlock_session(common_dir, key, key_source, repo_key_id)` -- persists
  the unlock session.
- `read_unlock_session(common_dir)` -- reads the current session, if any.
- `clear_unlock_session(common_dir)` -- removes the session file (locks the
  repo).

## Usage

Part of the [git-sshripped](https://github.com/BSteffaniak/git-sshripped)
workspace. This crate is not intended for standalone use.
