# git_sshripped_encryption

Deterministic file encryption and decryption for git-sshripped.

## Overview

This crate implements the core cryptographic operations that keep files
encrypted at rest in a Git repository. It uses AES-256-SIV for deterministic
authenticated encryption, deriving a per-file key from the repository key via
HKDF-SHA256. The file path is bound as authenticated associated data (AAD), so
a ciphertext only decrypts under the correct path.

Determinism is essential: the same key, path, and plaintext always produce the
same ciphertext, which allows Git to detect unchanged files and produce
meaningful diffs.

## Key Functions

- `is_encrypted(content)` -- checks for the `GSC1` magic prefix.
- `encrypt(algorithm, repo_key, path, plaintext)` -- encrypts plaintext,
  prepending the six-byte header. No-ops if the content is already encrypted.
- `decrypt(repo_key, path, encrypted)` -- parses the header and decrypts.

## Feature Flags

- `crypto-aes-siv` (default) -- enables the AES-SIV backend. Without this
  feature, encrypt/decrypt return an `UnsupportedAlgorithm` error.

## Usage

Part of the [git-sshripped](https://github.com/BSteffaniak/git-sshripped)
workspace. This crate is not intended for standalone use.
