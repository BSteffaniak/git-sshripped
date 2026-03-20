# git_sshripped_repository

Repository metadata persistence and Git integration for git-sshripped.

## Overview

This crate provides read/write operations for the `.git-sshripped/` metadata
directory. It handles TOML serialization of the repository manifest, GitHub
source registry, and local configuration. It also installs the `.gitattributes`
filter patterns and configures the local Git clean/smudge/diff/process filter
drivers via `git config`.

## Key Functions

- `metadata_dir(repo_root)` -- returns the `.git-sshripped/` path.
- `read_manifest()` / `write_manifest()` -- TOML persistence for the
  repository manifest.
- `read_github_sources()` / `write_github_sources()` -- TOML persistence for
  the GitHub source registry.
- `read_local_config()` / `write_local_config()` -- TOML persistence for
  local (non-committed) settings.
- `install_gitattributes(repo_root, patterns)` -- appends filter/diff
  attributes to `.gitattributes`.
- `install_git_filters(repo_root, bin)` -- sets up Git filter configuration
  via `git config --local`.

## Usage

Part of the [git-sshripped](https://github.com/BSteffaniak/git-sshripped)
workspace. This crate is not intended for standalone use.
