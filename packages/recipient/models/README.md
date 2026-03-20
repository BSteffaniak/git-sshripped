# git_sshripped_recipient_models

Data types for SSH key recipients in git-sshripped.

## Overview

This crate defines the serializable types that represent the SSH public-key
recipients authorized to unlock a repository. It contains no logic -- only the
shared vocabulary used by the recipient and CLI crates.

## Key Types

- `RecipientKey` -- a registered recipient with a SHA256 fingerprint, key type,
  full public-key line, and source.
- `RecipientSource` -- enum describing how a recipient was added: `LocalFile`,
  `LegacyGithubKeysUrl`, or `GithubKeys { url, username }`.

## Usage

Part of the [git-sshripped](https://github.com/BSteffaniak/git-sshripped)
workspace. This crate is not intended for standalone use.
