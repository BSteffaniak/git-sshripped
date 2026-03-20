# git_sshripped_cli_models

Data types for CLI initialization options in git-sshripped.

## Overview

This crate defines the serializable configuration types specific to the CLI's
`init` command. It is a small model crate that pairs an encryption algorithm
choice with the strict-mode flag.

## Key Types

- `InitOptions` -- struct with `algorithm: EncryptionAlgorithm` and
  `strict_mode: bool` (defaults to AES-SIV v1, non-strict).

## Usage

Part of the [git-sshripped](https://github.com/BSteffaniak/git-sshripped)
workspace. This crate is not intended for standalone use.
