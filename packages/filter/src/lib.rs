#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

use anyhow::{Result, bail};
use git_sshripped_encryption::{decrypt, encrypt, is_encrypted};
use git_sshripped_encryption_models::EncryptionAlgorithm;

/// Encrypt content for staging via the Git clean filter.
///
/// # Errors
///
/// Returns an error if the repository is locked (no key available) and the
/// content is unencrypted, or if encryption itself fails.
pub fn clean(
    algorithm: EncryptionAlgorithm,
    repo_key: Option<&[u8]>,
    path: &str,
    content: &[u8],
) -> Result<Vec<u8>> {
    profiling::scope!("clean");
    if is_encrypted(content) {
        return Ok(content.to_vec());
    }

    let key = repo_key.ok_or_else(|| {
        anyhow::anyhow!(
            "repository is locked and cannot encrypt protected file '{path}'; run git-sshripped unlock"
        )
    })?;
    encrypt(algorithm, key, path, content)
}

/// Decrypt content for checkout via the Git smudge filter.
///
/// # Errors
///
/// Returns an error if decryption fails. When the repository is locked,
/// ciphertext is passed through unchanged.
pub fn smudge(repo_key: Option<&[u8]>, path: &str, content: &[u8]) -> Result<Vec<u8>> {
    profiling::scope!("smudge");
    if !is_encrypted(content) {
        return Ok(content.to_vec());
    }

    if let Some(key) = repo_key {
        return decrypt(key, path, content);
    }

    Ok(content.to_vec())
}

/// Decrypt content for textconv diff display.
///
/// # Errors
///
/// Returns an error if the repository is locked and the content is encrypted,
/// or if decryption fails.
pub fn diff(repo_key: Option<&[u8]>, path: &str, content: &[u8]) -> Result<Vec<u8>> {
    profiling::scope!("diff");
    if !is_encrypted(content) {
        return Ok(content.to_vec());
    }

    if let Some(key) = repo_key {
        return decrypt(key, path, content);
    }

    bail!("file '{path}' is encrypted and repository is locked")
}
