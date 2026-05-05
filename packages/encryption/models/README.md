# git_sshripped_encryption_models

Pure data types for the git-sshripped encryption layer.

## Overview

This crate defines the serializable types that describe encrypted content and
the algorithms used to produce it. It contains no cryptographic logic -- only
the shared vocabulary consumed by the encryption, filter, repository, and CLI
crates.

## Key Types

- `ENCRYPTED_MAGIC` -- four-byte magic prefix (`GSC1`) that identifies
  encrypted content.
- `EncryptionAlgorithm` -- algorithm enum (`AesSivV1` legacy path-bound and
  `AesSivMovableV1` movable) with binary `id()`/`from_id()` round-tripping.
- `EncryptedHeader` -- the six-byte header (version + algorithm) prepended to
  every ciphertext blob.
- `EncryptionModelsError` -- error type for unknown algorithms or malformed
  headers.

## Usage

Part of the [git-sshripped](https://github.com/BSteffaniak/git-sshripped)
workspace. This crate is not intended for standalone use.
