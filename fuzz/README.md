# git_sshripped_fuzz

Fuzz testing targets for git-sshripped.

## Overview

This crate contains `cargo-fuzz` targets that test the robustness of
git-sshripped's encryption and protocol parsing against arbitrary input. It is
not a workspace member and is managed separately by cargo-fuzz.

## Fuzz Targets

- `encryption_decrypt` -- fuzzes the decryption path with random ciphertext
  and keys.
- `filter_pkt_protocol` -- fuzzes the Git filter long-running process protocol
  parser.

## Running

```sh
cargo +nightly fuzz run encryption_decrypt
cargo +nightly fuzz run filter_pkt_protocol
```
