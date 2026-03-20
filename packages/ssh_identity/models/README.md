# git_sshripped_ssh_identity_models

Data types for SSH identity sources in git-sshripped.

## Overview

This crate defines the serializable types that describe how an SSH identity was
discovered and which identity was used to unlock a repository. It contains no
logic -- only the shared vocabulary used by the ssh-identity and CLI crates.

## Key Types

- `IdentitySource` -- enum with variants `SshAgent` (key loaded in the SSH
  agent) and `IdentityFile` (key read from a file on disk).
- `IdentityDescriptor` -- pairs an `IdentitySource` with a human-readable
  label (e.g. a file path or agent key comment).

## Usage

Part of the [git-sshripped](https://github.com/BSteffaniak/git-sshripped)
workspace. This crate is not intended for standalone use.
