# git_sshripped_encryption

Deterministic file encryption and decryption for git-sshripped.

## Overview

This crate implements the core cryptographic operations that keep files
encrypted at rest in a Git repository. It uses AES-256-SIV for deterministic
authenticated encryption, deriving key material from the repository key via
HKDF-SHA256.

Newly encrypted files use the movable AES-SIV format by default: the associated
data (AAD) is a fixed git-sshripped domain string, so ciphertext can move to a
new repository-relative path without re-encryption.

The legacy `AesSivV1` format remains supported for path-bound ciphertext. In
that format, the repository-relative path is used as AAD and ciphertext only
decrypts under the same path.

Determinism is essential: the same key, algorithm, associated data, and
plaintext always produce the same ciphertext, which allows Git to detect
unchanged files and produce meaningful diffs.

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
