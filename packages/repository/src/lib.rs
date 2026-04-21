#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use git_sshripped_repository_models::{
    FilterInstallMarker, GithubSourceRegistry, RepositoryLocalConfig, RepositoryManifest,
};

#[must_use]
pub fn metadata_dir(repo_root: &Path) -> PathBuf {
    repo_root.join(".git-sshripped")
}

#[must_use]
pub fn manifest_file(repo_root: &Path) -> PathBuf {
    metadata_dir(repo_root).join("manifest.toml")
}

#[must_use]
pub fn github_sources_file(repo_root: &Path) -> PathBuf {
    metadata_dir(repo_root).join("github-sources.toml")
}

#[must_use]
pub fn local_config_file(repo_root: &Path) -> PathBuf {
    metadata_dir(repo_root).join("config.toml")
}

/// Write the repository manifest to `.git-sshripped/manifest.toml`.
///
/// # Errors
///
/// Returns an error if the metadata directory cannot be created, the manifest
/// cannot be serialized, or the file cannot be written.
pub fn write_manifest(repo_root: &Path, manifest: &RepositoryManifest) -> Result<()> {
    profiling::scope!("write_manifest");
    let dir = metadata_dir(repo_root);
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create metadata directory {}", dir.display()))?;
    let text = toml::to_string_pretty(manifest).context("failed to serialize manifest")?;
    let file = manifest_file(repo_root);
    fs::write(&file, text).with_context(|| format!("failed to write {}", file.display()))?;
    Ok(())
}

/// Read the repository manifest from `.git-sshripped/manifest.toml`.
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed.
pub fn read_manifest(repo_root: &Path) -> Result<RepositoryManifest> {
    profiling::scope!("read_manifest");
    let file = manifest_file(repo_root);
    let text = fs::read_to_string(&file)
        .with_context(|| format!("failed to read manifest {}", file.display()))?;
    toml::from_str(&text).context("failed to parse repository manifest")
}

/// Read the GitHub source registry, returning a default if the file does not exist.
///
/// # Errors
///
/// Returns an error if the file exists but cannot be read or parsed.
pub fn read_github_sources(repo_root: &Path) -> Result<GithubSourceRegistry> {
    let file = github_sources_file(repo_root);
    if !file.exists() {
        return Ok(GithubSourceRegistry::default());
    }
    let text = fs::read_to_string(&file)
        .with_context(|| format!("failed to read github source registry {}", file.display()))?;
    toml::from_str(&text).context("failed to parse github source registry")
}

/// Write the GitHub source registry to `.git-sshripped/github-sources.toml`.
///
/// # Errors
///
/// Returns an error if the metadata directory cannot be created, the registry
/// cannot be serialized, or the file cannot be written.
pub fn write_github_sources(repo_root: &Path, registry: &GithubSourceRegistry) -> Result<()> {
    let dir = metadata_dir(repo_root);
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create metadata directory {}", dir.display()))?;
    let text = toml::to_string_pretty(registry).context("failed to serialize github sources")?;
    let file = github_sources_file(repo_root);
    fs::write(&file, text)
        .with_context(|| format!("failed to write github source registry {}", file.display()))?;
    Ok(())
}

/// Read the local repository config, returning a default if the file does not exist.
///
/// # Errors
///
/// Returns an error if the file exists but cannot be read or parsed.
pub fn read_local_config(repo_root: &Path) -> Result<RepositoryLocalConfig> {
    let file = local_config_file(repo_root);
    if !file.exists() {
        return Ok(RepositoryLocalConfig::default());
    }
    let text = fs::read_to_string(&file)
        .with_context(|| format!("failed to read repository config {}", file.display()))?;
    toml::from_str(&text).context("failed to parse repository local config")
}

/// Write the local repository config to `.git-sshripped/config.toml`.
///
/// # Errors
///
/// Returns an error if the metadata directory cannot be created, the config
/// cannot be serialized, or the file cannot be written.
pub fn write_local_config(repo_root: &Path, config: &RepositoryLocalConfig) -> Result<()> {
    let dir = metadata_dir(repo_root);
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create metadata directory {}", dir.display()))?;
    let text = toml::to_string_pretty(config).context("failed to serialize local config")?;
    let file = local_config_file(repo_root);
    fs::write(&file, text)
        .with_context(|| format!("failed to write local config {}", file.display()))?;
    Ok(())
}

/// Append filter/diff attribute lines to `.gitattributes` for the given patterns.
///
/// # Errors
///
/// Returns an error if the `.gitattributes` file cannot be read or written.
pub fn install_gitattributes(repo_root: &Path, patterns: &[String]) -> Result<()> {
    profiling::scope!("install_gitattributes");
    let path = repo_root.join(".gitattributes");
    let mut existing = if path.exists() {
        fs::read_to_string(&path)
            .with_context(|| format!("failed to read gitattributes {}", path.display()))?
    } else {
        String::new()
    };

    for pattern in patterns {
        let line = pattern.strip_prefix('!').map_or_else(
            || format!("{pattern} filter=git-sshripped diff=git-sshripped"),
            |negated| format!("{negated} !filter !diff"),
        );
        if !existing.lines().any(|item| item.trim() == line) {
            if !existing.ends_with('\n') && !existing.is_empty() {
                existing.push('\n');
            }
            existing.push_str(&line);
            existing.push('\n');
        }
    }

    fs::write(&path, existing)
        .with_context(|| format!("failed to write gitattributes {}", path.display()))?;
    Ok(())
}

/// Shell-quote a string so it survives interpretation by the shell.
///
/// If the string contains no characters that need quoting it is returned as-is.
/// Otherwise it is wrapped in single quotes with any embedded single quotes
/// escaped using the `'\''` idiom.
fn shell_quote(s: &str) -> String {
    if !s.contains(|c: char| {
        c.is_whitespace()
            || matches!(
                c,
                '\'' | '"' | '\\' | '(' | ')' | '&' | ';' | '|' | '<' | '>' | '`' | '$' | '!' | '#'
            )
    }) {
        return s.to_string();
    }
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Install Git filter and diff configuration.
///
/// When `linked_worktree` is `true` the values are written to the
/// worktree-specific config layer (`git config --worktree`) so that a linked
/// worktree's binary path does not overwrite the shared repository config.
/// The `extensions.worktreeConfig` extension is enabled automatically when
/// needed.
///
/// For the main worktree (or when `linked_worktree` is `false`) the values
/// are written to the shared local config (`git config --local`) which acts
/// as the default fallback for all worktrees.
///
/// This function is **idempotent**: it runs `git config --get` for each key
/// first and only issues a write when the recorded value differs from the
/// desired one. On repositories where the filter config is already correct
/// (the common case when a user re-runs `unlock`), no writes happen at all.
///
/// # Errors
///
/// Returns an error if any `git config` command fails.
pub fn install_git_filters(repo_root: &Path, bin: &str, linked_worktree: bool) -> Result<()> {
    profiling::scope!("install_git_filters");
    // When writing to a linked worktree, ensure the worktreeConfig extension
    // is enabled (idempotent) so that `--worktree` scope is honoured by git.
    if linked_worktree {
        ensure_worktree_config_enabled(repo_root)?;
    }

    let scope = if linked_worktree {
        "--worktree"
    } else {
        "--local"
    };

    let quoted = shell_quote(bin);
    let pairs = [
        (
            "filter.git-sshripped.process".to_string(),
            format!("{quoted} filter-process"),
        ),
        (
            "filter.git-sshripped.clean".to_string(),
            format!("{quoted} clean --path %f"),
        ),
        (
            "filter.git-sshripped.smudge".to_string(),
            format!("{quoted} smudge --path %f"),
        ),
        (
            "filter.git-sshripped.required".to_string(),
            "true".to_string(),
        ),
        (
            "diff.git-sshripped.textconv".to_string(),
            format!("{quoted} diff --path %f"),
        ),
    ];

    for (key, value) in &pairs {
        ensure_git_config_value(repo_root, scope, key, value)?;
    }
    Ok(())
}

fn ensure_worktree_config_enabled(repo_root: &Path) -> Result<()> {
    const KEY: &str = "extensions.worktreeConfig";
    if git_config_read(repo_root, Some("--local"), KEY)?.as_deref() == Some("true") {
        return Ok(());
    }
    profiling::scope!("git config write", KEY);
    let ext_status = Command::new("git")
        .args(["config", "--local", KEY, "true"])
        .current_dir(repo_root)
        .status()
        .context("failed to enable extensions.worktreeConfig")?;
    if !ext_status.success() {
        anyhow::bail!("git config failed for key '{KEY}'");
    }
    Ok(())
}

fn ensure_git_config_value(repo_root: &Path, scope: &str, key: &str, value: &str) -> Result<()> {
    if git_config_read(repo_root, Some(scope), key)?.as_deref() == Some(value) {
        return Ok(());
    }
    profiling::scope!("git config write", key);
    let status = Command::new("git")
        .args(["config", scope, key, value])
        .current_dir(repo_root)
        .status()
        .with_context(|| format!("failed to set git config {key}"))?;
    if !status.success() {
        anyhow::bail!("git config failed for key '{key}'");
    }
    Ok(())
}

fn git_config_read(repo_root: &Path, scope: Option<&str>, key: &str) -> Result<Option<String>> {
    profiling::scope!("git config read", key);
    let mut cmd = Command::new("git");
    cmd.current_dir(repo_root).arg("config");
    if let Some(s) = scope {
        cmd.arg(s);
    }
    cmd.args(["--get", key]);
    let output = cmd
        .output()
        .with_context(|| format!("failed to run git config --get {key}"))?;
    if !output.status.success() {
        // Exit code 1 means the key is unset; that's not an error.
        return Ok(None);
    }
    let text = String::from_utf8(output.stdout)
        .with_context(|| format!("git config value for {key} is not utf8"))?
        .trim_end_matches('\n')
        .to_string();
    Ok(Some(text))
}

// ---------------------------------------------------------------------------
// Agent-wrapped key file helpers
//
// Agent-wrap files are stored inside the git common directory
// (`git rev-parse --git-common-dir`) so they are:
//   - local to the machine (never committed)
//   - shared across linked worktrees
// ---------------------------------------------------------------------------

/// Directory for agent-wrapped key files inside the git common directory.
#[must_use]
pub fn agent_wrap_dir(common_dir: &Path) -> PathBuf {
    common_dir.join("git-sshripped-agent-wrap")
}

/// Path to an agent-wrapped key file for a given fingerprint.
#[must_use]
pub fn agent_wrap_file(common_dir: &Path, fingerprint: &str) -> PathBuf {
    agent_wrap_dir(common_dir).join(format!("{fingerprint}.toml"))
}

/// Read an agent-wrapped key file, returning `None` if the file does not exist.
///
/// # Errors
///
/// Returns an error if the file exists but cannot be read or parsed.
pub fn read_agent_wrap(
    common_dir: &Path,
    fingerprint: &str,
) -> Result<Option<git_sshripped_ssh_agent_models::AgentWrappedKey>> {
    profiling::scope!("read_agent_wrap");
    let file = agent_wrap_file(common_dir, fingerprint);
    if !file.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(&file)
        .with_context(|| format!("failed to read agent-wrap file {}", file.display()))?;
    let key: git_sshripped_ssh_agent_models::AgentWrappedKey =
        toml::from_str(&text).context("failed to parse agent-wrap file")?;
    Ok(Some(key))
}

/// Write an agent-wrapped key file.
///
/// # Errors
///
/// Returns an error if the directory cannot be created, the file cannot be
/// serialized, or the file cannot be written.
pub fn write_agent_wrap(
    common_dir: &Path,
    wrapped: &git_sshripped_ssh_agent_models::AgentWrappedKey,
) -> Result<()> {
    profiling::scope!("write_agent_wrap");
    let dir = agent_wrap_dir(common_dir);
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create agent-wrap directory {}", dir.display()))?;
    let file = agent_wrap_file(common_dir, &wrapped.fingerprint);
    let text = toml::to_string_pretty(wrapped).context("failed to serialize agent-wrap key")?;
    fs::write(&file, text)
        .with_context(|| format!("failed to write agent-wrap file {}", file.display()))?;
    Ok(())
}

/// List all agent-wrap `.toml` files in the agent-wrap directory.
///
/// # Errors
///
/// Returns an error if the directory cannot be read.
pub fn list_agent_wrap_files(common_dir: &Path) -> Result<Vec<PathBuf>> {
    profiling::scope!("list_agent_wrap_files");
    let dir = agent_wrap_dir(common_dir);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for entry in fs::read_dir(&dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("toml"))
        {
            files.push(path);
        }
    }
    Ok(files)
}

/// Parse an agent-wrapped key from a TOML string.
///
/// # Errors
///
/// Returns an error if the string is not valid TOML or does not match the
/// expected schema.
pub fn parse_agent_wrap(text: &str) -> Result<git_sshripped_ssh_agent_models::AgentWrappedKey> {
    toml::from_str(text).context("failed to parse agent-wrap TOML")
}

// ---------------------------------------------------------------------------
// Filter install marker
//
// The marker lives alongside the unlock session in the git common directory
// so it is:
//   - local to the machine (never committed)
//   - shared across linked worktrees for the main-worktree case
//   - discarded whenever the common dir is nuked
// ---------------------------------------------------------------------------

/// Path to the filter-install marker for a given common dir.
#[must_use]
pub fn filter_marker_file(common_dir: &Path) -> PathBuf {
    common_dir
        .join("git-sshripped")
        .join("session")
        .join("filters-installed.json")
}

/// Read the filter-install marker if present.
///
/// Returns `None` when the marker is absent or malformed. A malformed marker
/// is not fatal -- callers treat it the same as a missing marker and fall
/// through to a full reinstall.
#[must_use]
pub fn read_filter_marker(common_dir: &Path) -> Option<FilterInstallMarker> {
    profiling::scope!("read_filter_marker");
    let path = filter_marker_file(common_dir);
    let text = fs::read_to_string(&path).ok()?;
    serde_json::from_str::<FilterInstallMarker>(&text).ok()
}

/// Persist the filter-install marker.
///
/// Any I/O or serialization failure is non-fatal -- the marker is a cache
/// hint, not a source of truth. Callers therefore get `()` back and do not
/// propagate errors from this function.
pub fn write_filter_marker(common_dir: &Path, marker: &FilterInstallMarker) {
    profiling::scope!("write_filter_marker");
    let path = filter_marker_file(common_dir);
    let Some(parent) = path.parent() else {
        return;
    };
    if fs::create_dir_all(parent).is_err() {
        return;
    }
    if let Ok(text) = serde_json::to_string_pretty(marker) {
        let _ = fs::write(&path, text);
    }
}

/// Remove the filter-install marker, if present.
///
/// Callers use this when they know filter configuration has been touched
/// outside of [`install_git_filters`] (e.g. the `install` subcommand or any
/// direct `git config --unset` performed elsewhere).
pub fn clear_filter_marker(common_dir: &Path) {
    profiling::scope!("clear_filter_marker");
    let path = filter_marker_file(common_dir);
    let _ = fs::remove_file(path);
}
