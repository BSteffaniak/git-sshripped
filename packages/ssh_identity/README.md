# git_sshripped_ssh_identity

SSH key discovery and repo-key unwrapping for git-sshripped.

## Overview

This crate discovers SSH private keys that can unlock a repository and performs
the actual decryption of the wrapped repo key. It supports multiple key
discovery strategies:

- **File scanning** -- enumerates key pairs in `~/.ssh/` and parses
  `~/.ssh/config` for `IdentityFile` directives.
- **SSH agent** -- queries the running SSH agent for loaded public keys and
  matches them against local private key files.
- **Agent helper** -- delegates to an external helper program for custom
  unwrapping workflows.

Once an identity is selected, the crate uses the `age` library to decrypt the
age-encrypted wrapped key file, prompting for passphrases when needed (via
`rpassword` or the `GSC_SSH_KEY_PASSPHRASE` environment variable).

## Key Functions

- `discover_ssh_key_files()` / `default_private_key_candidates()` /
  `well_known_public_key_paths()` -- key file discovery.
- `agent_public_keys()` / `private_keys_matching_agent()` -- SSH agent
  integration.
- `detect_identity()` -- auto-detects the best available SSH identity.
- `unwrap_repo_key_from_wrapped_files()` -- tries each identity to decrypt a
  wrapped key file.
- `unwrap_repo_key_with_agent_helper()` -- delegates to an external helper.

## Usage

Part of the [git-sshripped](https://github.com/BSteffaniak/git-sshripped)
workspace. This crate is not intended for standalone use.
