# git_sshripped_filter_models

Data types for the git-sshripped Git filter layer.

## Overview

This crate defines the `FilterOperation` enum that represents the two
directions of a Git clean/smudge filter. It is a minimal model crate with a
single serde-serializable type.

## Key Types

- `FilterOperation` -- enum with variants `Clean` (encrypt on stage) and
  `Smudge` (decrypt on checkout).

## Usage

Part of the [git-sshripped](https://github.com/BSteffaniak/git-sshripped)
workspace. This crate is not intended for standalone use.
