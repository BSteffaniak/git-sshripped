# git_sshripped_cli

The primary command-line interface for git-sshripped.

## Overview

This crate implements the `git-sshripped` CLI, wiring together every domain
crate in the workspace. It uses `clap` for argument parsing and exposes a
single `run()` entry point that the binary (and alias) crates call.

## Subcommands

| Command | Description |
|---------|-------------|
| `init` | Initialize encryption with file patterns, algorithm, and recipients |
| `unlock` / `lock` | Unlock or lock the repository |
| `status` | Show lock/unlock state and repository info |
| `doctor` | Diagnose configuration issues |
| `verify` | Verify encryption integrity |
| `rewrap` | Re-wrap the repo key for all current recipients |
| `rotate-key` | Generate a new repo key with optional auto-reencrypt |
| `reencrypt` | Re-encrypt all protected files with the current key |
| `add-user` / `remove-user` / `list-users` | Manage individual recipients |
| `add-github-user` / `remove-github-user` | Manage GitHub-sourced recipients |
| `add-github-team` / `remove-github-team` | Manage GitHub team recipients |
| `refresh-github-keys` / `refresh-github-teams` | Refresh cached GitHub keys |
| `revoke-user` | Revoke a user with optional auto-reencrypt |
| `access-audit` | Audit which identities can access the repo |
| `install` | Re-install Git filter configuration |
| `migrate-from-git-crypt` | Migrate from git-crypt |
| `export-repo-key` / `import-repo-key` | Export or import the raw data key |
| `policy` | Manage security policy settings |
| `config` | Manage local configuration |

The `clean`, `smudge`, `diff`, and `filter-process` subcommands are low-level
entry points invoked by Git's filter driver.

## Usage

Part of the [git-sshripped](https://github.com/BSteffaniak/git-sshripped)
workspace. The binary is built as `git-sshripped` and can also be invoked via
the `git-sshript` and `git-sshrypt` alias binaries.
