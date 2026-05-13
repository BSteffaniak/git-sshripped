//! Blob-hash → repo-relative path resolver for the diff/textconv code path.
//!
//! Git's textconv contract (`diff.<driver>.textconv = …`) hands the textconv
//! command a temp file containing the blob being converted to text, **and
//! nothing else** — no path, no commit OID, no `%f` substitution. Our
//! encryption layer needs the original repo-relative path as AAD when
//! decrypting path-bound (`AesSivV1`) ciphertext, so the textconv must
//! reverse-resolve the path from the blob bytes.
//!
//! This module computes the git blob OID of the supplied bytes (via
//! `git hash-object --stdin`, which automatically matches the repository's
//! chosen object format — SHA-1 or SHA-256) and returns every repo-relative
//! path that has a matching OID across the union of:
//!
//! * all reachable history (`git rev-list --objects --all`)
//! * the current index (`git ls-files -s`)
//!
//! Results are cached on disk at `<git-dir>/git-sshripped/textconv-paths.cache`
//! keyed by the current `HEAD` revision. The cache is regenerated when HEAD
//! changes, and on a lookup miss against an existing cache we force one
//! rebuild before giving up (covers `git fetch` adding new refs without
//! moving HEAD).
//!
//! ## Infallibility contract
//!
//! [`resolve_paths`] is infallible by signature: any internal failure
//! (subprocess error, missing git, no commits, unwritable git-dir, malformed
//! cache, non-UTF-8 paths, shallow clone, …) collapses to `Vec::new()`. The
//! diff/textconv code path runs during `git status`, `git diff`, `git log -p`,
//! `git show`, and `git blame`; a hard failure here would block those Git
//! operations the same way the original `b021580` regression did. Every
//! Result-returning helper in this module either swallows its error to a
//! log-and-continue or returns `Result` only internally where the outer
//! `resolve_paths` is guaranteed to map it to `Vec::new()`.
//!
//! Cache contents are intentionally non-secret: object IDs and repo paths
//! are already in git's own object database. Encryption keys, plaintext, and
//! recipient material never touch this cache.

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const CACHE_DIR_NAME: &str = "git-sshripped";
const CACHE_FILE_NAME: &str = "textconv-paths.cache";
const HEAD_NONE_SENTINEL: &str = "NONE";

/// Resolve candidate repo-relative paths for an encrypted blob.
///
/// Infallible by contract. Any internal failure collapses to an empty
/// `Vec`, which callers must treat as "no candidates found" without
/// distinguishing it from "lookup unavailable".
#[must_use]
pub fn resolve_paths(repo_root: &Path, content: &[u8]) -> Vec<String> {
    profiling::scope!("textconv_paths::resolve_paths");
    let Some(blob_hash) = git_blob_hash(repo_root, content) else {
        return Vec::new();
    };
    let head = current_head(repo_root);

    let Some(cache_path) = cache_file_path(repo_root) else {
        // No git-dir resolvable → fall back to a one-shot in-memory
        // rebuild without persistence.
        return rebuild_map(repo_root)
            .remove(&blob_hash)
            .unwrap_or_default();
    };

    if let Some(map) = load_cache(&cache_path, head.as_deref())
        && let Some(paths) = map.get(&blob_hash)
    {
        return paths.clone();
    }

    // Cache miss (either absent, stale, or doesn't contain the OID). Rebuild
    // once, persist best-effort, retry lookup, then give up.
    let map = rebuild_map(repo_root);
    write_cache_best_effort(&cache_path, head.as_deref(), &map);
    map.get(&blob_hash).cloned().unwrap_or_default()
}

/// Compute the git blob OID of `content`. Uses `git hash-object --stdin` so
/// the hash matches whatever object format the repository uses (SHA-1 or
/// SHA-256). Returns `None` on any failure.
fn git_blob_hash(repo_root: &Path, content: &[u8]) -> Option<String> {
    profiling::scope!("textconv_paths::git_blob_hash");
    let mut child = Command::new("git")
        .current_dir(repo_root)
        .args(["hash-object", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    {
        let stdin = child.stdin.as_mut()?;
        stdin.write_all(content).ok()?;
    }
    let output = child.wait_with_output().ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8(output.stdout).ok()?;
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn current_head(repo_root: &Path) -> Option<String> {
    profiling::scope!("textconv_paths::current_head");
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["rev-parse", "HEAD"])
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8(output.stdout).ok()?;
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn cache_file_path(repo_root: &Path) -> Option<PathBuf> {
    let git_dir = git_dir(repo_root)?;
    Some(git_dir.join(CACHE_DIR_NAME).join(CACHE_FILE_NAME))
}

fn git_dir(repo_root: &Path) -> Option<PathBuf> {
    profiling::scope!("textconv_paths::git_dir");
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["rev-parse", "--git-dir"])
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8(output.stdout).ok()?;
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return None;
    }
    let p = PathBuf::from(trimmed);
    if p.is_absolute() {
        Some(p)
    } else {
        Some(repo_root.join(p))
    }
}

fn load_cache(path: &Path, expected_head: Option<&str>) -> Option<HashMap<String, Vec<String>>> {
    profiling::scope!("textconv_paths::load_cache");
    let text = fs::read_to_string(path).ok()?;
    let mut lines = text.lines();
    let stored_head = lines.next()?;
    let expected = expected_head.unwrap_or(HEAD_NONE_SENTINEL);
    if stored_head != expected {
        return None;
    }
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for line in lines {
        if let Some((oid, p)) = line.split_once(' ') {
            if oid.is_empty() || p.is_empty() {
                continue;
            }
            map.entry(oid.to_string()).or_default().push(p.to_string());
        }
    }
    Some(map)
}

fn write_cache_best_effort(path: &Path, head: Option<&str>, map: &HashMap<String, Vec<String>>) {
    profiling::scope!("textconv_paths::write_cache_best_effort");
    if let Some(parent) = path.parent()
        && fs::create_dir_all(parent).is_err()
    {
        return;
    }
    let mut buf = String::new();
    buf.push_str(head.unwrap_or(HEAD_NONE_SENTINEL));
    buf.push('\n');
    for (oid, paths) in map {
        for p in paths {
            // Skip entries containing newlines (would corrupt the line-based
            // format); these are extraordinarily rare in practice.
            if oid.contains('\n') || p.contains('\n') {
                continue;
            }
            buf.push_str(oid);
            buf.push(' ');
            buf.push_str(p);
            buf.push('\n');
        }
    }
    let _ = fs::write(path, buf);
}

fn rebuild_map(repo_root: &Path) -> HashMap<String, Vec<String>> {
    profiling::scope!("textconv_paths::rebuild_map");
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    parse_rev_list_into(repo_root, &mut map);
    parse_ls_files_into(repo_root, &mut map);
    map
}

fn parse_rev_list_into(repo_root: &Path, sink: &mut HashMap<String, Vec<String>>) {
    profiling::scope!("textconv_paths::parse_rev_list_into");
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["rev-list", "--objects", "--all"])
        .stderr(Stdio::null())
        .output();
    let Ok(output) = output else { return };
    if !output.status.success() {
        return;
    }
    let Ok(text) = String::from_utf8(output.stdout) else {
        return;
    };
    for line in text.lines() {
        // Lines are either `<oid>` (commits/tags/trees with no path) or
        // `<oid> <path>` (blobs and trees with their path). We only want
        // the latter.
        if let Some((oid, path)) = line.split_once(' ') {
            if oid.is_empty() || path.is_empty() {
                continue;
            }
            sink.entry(oid.to_string())
                .or_default()
                .push(path.to_string());
        }
    }
}

fn parse_ls_files_into(repo_root: &Path, sink: &mut HashMap<String, Vec<String>>) {
    profiling::scope!("textconv_paths::parse_ls_files_into");
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["ls-files", "-s"])
        .stderr(Stdio::null())
        .output();
    let Ok(output) = output else { return };
    if !output.status.success() {
        return;
    }
    let Ok(text) = String::from_utf8(output.stdout) else {
        return;
    };
    for line in text.lines() {
        // Format: `<mode> <oid> <stage>\t<path>`
        let Some((meta, path)) = line.split_once('\t') else {
            continue;
        };
        let parts: Vec<&str> = meta.split_whitespace().collect();
        let Some(oid) = parts.get(1) else { continue };
        if oid.is_empty() || path.is_empty() {
            continue;
        }
        sink.entry((*oid).to_string())
            .or_default()
            .push(path.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    /// Run `git init` in a fresh tempdir, return the path. Skips on systems
    /// without git in PATH.
    fn init_repo() -> Option<(tempfile::TempDir, PathBuf)> {
        let dir = tempfile::TempDir::new().ok()?;
        let status = Command::new("git")
            .current_dir(dir.path())
            .args(["init", "-q"])
            .status()
            .ok()?;
        if !status.success() {
            return None;
        }
        let _ = Command::new("git")
            .current_dir(dir.path())
            .args(["config", "user.email", "test@example.com"])
            .status();
        let _ = Command::new("git")
            .current_dir(dir.path())
            .args(["config", "user.name", "test"])
            .status();
        let path = dir.path().to_path_buf();
        Some((dir, path))
    }

    #[test]
    fn resolve_paths_returns_empty_for_non_repo() {
        // Pointing at a directory that is not a git repo must never panic and
        // must return an empty Vec.
        let dir = tempfile::TempDir::new().expect("temp dir");
        let result = resolve_paths(dir.path(), b"some content");
        assert!(result.is_empty());
    }

    #[test]
    fn resolve_paths_returns_empty_for_unknown_blob() {
        let Some((_guard, repo)) = init_repo() else {
            eprintln!("git not available; skipping");
            return;
        };
        let result = resolve_paths(&repo, b"never-stored content");
        assert!(result.is_empty());
    }

    #[test]
    fn resolve_paths_finds_committed_blob() {
        let Some((_guard, repo)) = init_repo() else {
            eprintln!("git not available; skipping");
            return;
        };
        let plaintext = b"hello world\n";
        fs::write(repo.join("README.md"), plaintext).expect("write");
        Command::new("git")
            .current_dir(&repo)
            .args(["add", "README.md"])
            .status()
            .expect("git add");
        Command::new("git")
            .current_dir(&repo)
            .args(["commit", "-q", "-m", "initial"])
            .status()
            .expect("git commit");
        let result = resolve_paths(&repo, plaintext);
        assert!(
            result.iter().any(|p| p == "README.md"),
            "expected README.md in {result:?}"
        );
    }

    #[test]
    fn resolve_paths_finds_staged_only_blob() {
        let Some((_guard, repo)) = init_repo() else {
            eprintln!("git not available; skipping");
            return;
        };
        let plaintext = b"staged but not committed\n";
        fs::write(repo.join("staged.txt"), plaintext).expect("write");
        Command::new("git")
            .current_dir(&repo)
            .args(["add", "staged.txt"])
            .status()
            .expect("git add");
        let result = resolve_paths(&repo, plaintext);
        assert!(
            result.iter().any(|p| p == "staged.txt"),
            "expected staged.txt in {result:?}"
        );
    }

    #[test]
    fn cache_is_invalidated_when_head_changes() {
        let Some((_guard, repo)) = init_repo() else {
            eprintln!("git not available; skipping");
            return;
        };
        let plaintext_a = b"first\n";
        fs::write(repo.join("a.txt"), plaintext_a).expect("write a");
        Command::new("git")
            .current_dir(&repo)
            .args(["add", "a.txt"])
            .status()
            .expect("git add a");
        Command::new("git")
            .current_dir(&repo)
            .args(["commit", "-q", "-m", "a"])
            .status()
            .expect("git commit a");
        // Populate cache.
        let _ = resolve_paths(&repo, plaintext_a);
        let cache = cache_file_path(&repo).expect("cache path");
        assert!(cache.exists(), "cache should be written");
        let initial = fs::read_to_string(&cache).expect("read cache");
        let initial_head = initial.lines().next().expect("first line").to_string();

        // Move HEAD with a new commit.
        let plaintext_b = b"second\n";
        fs::write(repo.join("b.txt"), plaintext_b).expect("write b");
        Command::new("git")
            .current_dir(&repo)
            .args(["add", "b.txt"])
            .status()
            .expect("git add b");
        Command::new("git")
            .current_dir(&repo)
            .args(["commit", "-q", "-m", "b"])
            .status()
            .expect("git commit b");

        // Look up the new blob — should rebuild and find it.
        let result = resolve_paths(&repo, plaintext_b);
        assert!(
            result.iter().any(|p| p == "b.txt"),
            "expected b.txt in {result:?}"
        );
        let updated = fs::read_to_string(&cache).expect("read cache 2");
        let updated_head = updated.lines().next().expect("first line 2").to_string();
        assert_ne!(initial_head, updated_head, "cache HEAD should be refreshed");
    }

    #[test]
    fn corrupt_cache_is_treated_as_miss() {
        let Some((_guard, repo)) = init_repo() else {
            eprintln!("git not available; skipping");
            return;
        };
        fs::write(repo.join("c.txt"), b"c\n").expect("write");
        Command::new("git")
            .current_dir(&repo)
            .args(["add", "c.txt"])
            .status()
            .expect("git add");
        Command::new("git")
            .current_dir(&repo)
            .args(["commit", "-q", "-m", "c"])
            .status()
            .expect("git commit");
        let cache = cache_file_path(&repo).expect("cache path");
        if let Some(parent) = cache.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&cache, b"\x00\x01garbage not utf8\xff\xfe").expect("write garbage");
        // Must not panic; must still find c.txt by rebuilding.
        let result = resolve_paths(&repo, b"c\n");
        assert!(
            result.iter().any(|p| p == "c.txt"),
            "expected c.txt in {result:?}"
        );
    }
}
