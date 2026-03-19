#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

use std::fs;
use std::io::Write;
use std::iter;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use age::Encryptor;
use age::ssh::Recipient as SshRecipient;
use anyhow::{Context, Result, bail};
use base64::Engine;
use git_ssh_crypt_recipient_models::{RecipientKey, RecipientSource};
use sha2::{Digest, Sha256};

const SUPPORTED_KEY_TYPES: [&str; 2] = ["ssh-ed25519", "ssh-rsa"];

#[must_use]
pub fn recipient_store_dir(repo_root: &Path) -> PathBuf {
    repo_root.join(".git-ssh-crypt").join("recipients")
}

#[must_use]
pub fn wrapped_store_dir(repo_root: &Path) -> PathBuf {
    repo_root.join(".git-ssh-crypt").join("wrapped")
}

pub fn list_recipients(repo_root: &Path) -> Result<Vec<RecipientKey>> {
    let dir = recipient_store_dir(repo_root);
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut recipients = Vec::new();
    for entry in fs::read_dir(&dir)
        .with_context(|| format!("failed to read recipient dir {}", dir.display()))?
    {
        let entry = entry.with_context(|| format!("failed to read entry in {}", dir.display()))?;
        if !entry
            .file_type()
            .with_context(|| format!("failed to read entry type for {}", entry.path().display()))?
            .is_file()
        {
            continue;
        }
        let text = fs::read_to_string(entry.path())
            .with_context(|| format!("failed to read recipient file {}", entry.path().display()))?;
        let recipient: RecipientKey = toml::from_str(&text).with_context(|| {
            format!("failed to parse recipient file {}", entry.path().display())
        })?;
        recipients.push(recipient);
    }

    recipients.sort_by(|a, b| a.fingerprint.cmp(&b.fingerprint));
    Ok(recipients)
}

pub fn add_recipient_from_public_key(
    repo_root: &Path,
    public_key_line: &str,
    source: RecipientSource,
) -> Result<RecipientKey> {
    let trimmed = public_key_line.trim();
    if trimmed.is_empty() {
        bail!("empty SSH public key line");
    }

    let mut parts = trimmed.split_whitespace();
    let key_type = parts
        .next()
        .context("SSH public key is missing key type")?
        .to_string();
    let key_body = parts
        .next()
        .context("SSH public key is missing key material")?;

    if !SUPPORTED_KEY_TYPES
        .iter()
        .any(|supported| *supported == key_type)
    {
        bail!(
            "unsupported SSH key type '{key_type}'; supported types: {}",
            SUPPORTED_KEY_TYPES.join(", ")
        );
    }

    let mut hasher = Sha256::new();
    hasher.update(key_type.as_bytes());
    hasher.update([b':']);
    hasher.update(key_body.as_bytes());
    let fingerprint = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hasher.finalize());

    let recipient = RecipientKey {
        fingerprint: fingerprint.clone(),
        key_type,
        public_key_line: trimmed.to_string(),
        source,
    };

    let dir = recipient_store_dir(repo_root);
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create recipient dir {}", dir.display()))?;
    let file = dir.join(format!("{fingerprint}.toml"));
    let content = toml::to_string_pretty(&recipient)
        .with_context(|| format!("failed to serialize recipient {}", recipient.fingerprint))?;
    fs::write(&file, content)
        .with_context(|| format!("failed to write recipient file {}", file.display()))?;

    Ok(recipient)
}

pub fn add_recipients_from_github_keys(repo_root: &Path, url: &str) -> Result<Vec<RecipientKey>> {
    add_recipients_from_github_source(repo_root, url, None)
}

pub fn add_recipients_from_github_username(
    repo_root: &Path,
    username: &str,
) -> Result<Vec<RecipientKey>> {
    let url = format!("https://github.com/{username}.keys");
    add_recipients_from_github_source(repo_root, &url, Some(username.to_string()))
}

pub fn add_recipients_from_github_source(
    repo_root: &Path,
    url: &str,
    username: Option<String>,
) -> Result<Vec<RecipientKey>> {
    let text = reqwest::blocking::get(url)
        .with_context(|| format!("failed to GET {url}"))?
        .error_for_status()
        .with_context(|| format!("GitHub keys request returned error for {url}"))?
        .text()
        .context("failed to read GitHub keys body")?;

    let mut added = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let recipient = add_recipient_from_public_key(
            repo_root,
            line,
            RecipientSource::GithubKeys {
                url: url.to_string(),
                username: username.clone(),
            },
        )
        .with_context(|| format!("failed to add recipient from key line '{line}'"))?;
        added.push(recipient);
    }

    Ok(added)
}

pub fn remove_recipients_by_fingerprints(
    repo_root: &Path,
    fingerprints: &[String],
) -> Result<usize> {
    let mut removed = 0;
    for fingerprint in fingerprints {
        if remove_recipient_by_fingerprint(repo_root, fingerprint)? {
            removed += 1;
        }
    }
    Ok(removed)
}

pub fn wrap_repo_key_for_recipient(
    repo_root: &Path,
    recipient: &RecipientKey,
    repo_key: &[u8],
) -> Result<PathBuf> {
    let ssh_recipient = SshRecipient::from_str(&recipient.public_key_line).map_err(|err| {
        anyhow::anyhow!(
            "invalid ssh public key for {}: {:?}",
            recipient.fingerprint,
            err
        )
    })?;

    let encryptor = Encryptor::with_recipients(iter::once(&ssh_recipient as _))
        .context("failed to initialize age encryptor")?;

    let mut wrapped = Vec::new();
    {
        let mut writer = encryptor
            .wrap_output(&mut wrapped)
            .context("failed to start age wrapping")?;
        writer
            .write_all(repo_key)
            .context("failed to write repo key to wrapper")?;
        writer.finish().context("failed to finish age wrapping")?;
    }

    let dir = wrapped_store_dir(repo_root);
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create wrapped dir {}", dir.display()))?;
    let wrapped_file = dir.join(format!("{}.age", recipient.fingerprint));
    fs::write(&wrapped_file, wrapped)
        .with_context(|| format!("failed to write wrapped key {}", wrapped_file.display()))?;
    Ok(wrapped_file)
}

pub fn wrap_repo_key_for_all_recipients(repo_root: &Path, repo_key: &[u8]) -> Result<Vec<PathBuf>> {
    let recipients = list_recipients(repo_root)?;
    if recipients.is_empty() {
        bail!("no recipients configured; add at least one recipient first");
    }

    let mut wrapped_files = Vec::new();
    for recipient in recipients {
        let wrapped_file = wrap_repo_key_for_recipient(repo_root, &recipient, repo_key)?;
        wrapped_files.push(wrapped_file);
    }
    Ok(wrapped_files)
}

pub fn remove_recipient_by_fingerprint(repo_root: &Path, fingerprint: &str) -> Result<bool> {
    let recipient_file = recipient_store_dir(repo_root).join(format!("{fingerprint}.toml"));
    let wrapped_file = wrapped_store_dir(repo_root).join(format!("{fingerprint}.age"));

    let mut removed_any = false;
    if recipient_file.exists() {
        fs::remove_file(&recipient_file).with_context(|| {
            format!(
                "failed to remove recipient file {}",
                recipient_file.display()
            )
        })?;
        removed_any = true;
    }
    if wrapped_file.exists() {
        fs::remove_file(&wrapped_file)
            .with_context(|| format!("failed to remove wrapped file {}", wrapped_file.display()))?;
        removed_any = true;
    }

    Ok(removed_any)
}
