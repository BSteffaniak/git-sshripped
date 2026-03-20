# git_sshripped_repository_models

Configuration and manifest data types for git-sshripped.

## Overview

This crate defines the serializable configuration types that live in the
`.git-sshripped/` metadata directory. It covers the repository manifest,
GitHub source registry, and local-only configuration -- all as plain structs
with serde support.

## Key Types

- `RepositoryManifest` -- the main per-repo config: encryption algorithm,
  strict mode, `repo_key_id`, minimum recipients, allowed key types, rotation
  and revocation policy flags, and `max_source_staleness_hours`.
- `GithubSourceRegistry` -- tracks GitHub users and teams whose SSH keys are
  fetched as recipients. Contains `Vec<GithubUserSource>` and
  `Vec<GithubTeamSource>`.
- `GithubUserSource` / `GithubTeamSource` -- per-source metadata including
  URLs, fingerprints, refresh timestamps, and ETags for conditional requests.
- `RepositoryLocalConfig` -- non-committed local settings: agent helper path,
  GitHub API/web base URLs, auth mode, and private-source hard-fail flag.

## Usage

Part of the [git-sshripped](https://github.com/BSteffaniak/git-sshripped)
workspace. This crate is not intended for standalone use.
