//! Criterion benches for `git-sshripped unlock --soft` on the
//! already-unlocked fast path.
//!
//! Real-repo benches are opt-in via environment variables so CI runs stay
//! cheap:
//!
//! * `BENCH_REPO` — absolute path to a pre-unlocked git-sshripped repository
//!   (e.g. `~/GitHub/monorepo`).
//! * `GIT_SSHRIPPED_BIN` — absolute path to the release binary built with
//!   `--features profile-trace` (or any release build for raw timing).
//!
//! If either variable is missing the benches that depend on it print a
//! `skipping` notice and return early.

use std::path::PathBuf;
use std::process::Command;

use criterion::{Criterion, criterion_group, criterion_main};

fn bench_repo() -> Option<PathBuf> {
    std::env::var_os("BENCH_REPO").map(PathBuf::from)
}

fn bench_bin() -> Option<PathBuf> {
    std::env::var_os("GIT_SSHRIPPED_BIN").map(PathBuf::from)
}

fn bench_git_rev_parse(c: &mut Criterion) {
    let Some(repo) = bench_repo() else {
        eprintln!("baseline/git_rev_parse: BENCH_REPO unset; skipping");
        return;
    };

    c.bench_function("baseline/git_rev_parse", |b| {
        b.iter(|| {
            let out = Command::new("git")
                .args(["rev-parse", "--show-toplevel"])
                .current_dir(&repo)
                .output()
                .expect("failed to spawn git");
            assert!(out.status.success());
        });
    });
}

fn bench_sshripped_version(c: &mut Criterion) {
    let Some(bin) = bench_bin() else {
        eprintln!("baseline/sshripped_version: GIT_SSHRIPPED_BIN unset; skipping");
        return;
    };

    c.bench_function("baseline/sshripped_version", |b| {
        b.iter(|| {
            let out = Command::new(&bin)
                .arg("--version")
                .output()
                .expect("failed to spawn sshripped");
            assert!(out.status.success());
        });
    });
}

fn bench_unlock_process(c: &mut Criterion) {
    let (Some(repo), Some(bin)) = (bench_repo(), bench_bin()) else {
        eprintln!(
            "unlock_already_unlocked/process: BENCH_REPO and GIT_SSHRIPPED_BIN required; skipping"
        );
        return;
    };

    c.bench_function("unlock_already_unlocked/process", |b| {
        b.iter(|| {
            let out = Command::new(&bin)
                .arg("unlock")
                .arg("--soft")
                .current_dir(&repo)
                .output()
                .expect("failed to spawn sshripped");
            assert!(
                out.status.success(),
                "unlock failed: stdout={} stderr={}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr),
            );
        });
    });
}

fn bench_unlock_in_process(c: &mut Criterion) {
    let Some(repo) = bench_repo() else {
        eprintln!("unlock_already_unlocked/in_process: BENCH_REPO unset; skipping");
        return;
    };

    // Pre-flight: ensure the repo is unlocked by shelling out once. If the
    // bench ran in an already-unlocked steady state we still want to be sure.
    if let Some(bin) = bench_bin() {
        let _ = Command::new(&bin)
            .arg("unlock")
            .arg("--soft")
            .current_dir(&repo)
            .output();
    }

    c.bench_function("unlock_already_unlocked/in_process", |b| {
        b.iter(|| {
            // The bench hook mirrors `cmd_unlock` with no-arg invocation
            // semantics (no key hex, no identities, no github user, prefer
            // agent false, no-agent false). It internally calls
            // `std::env::current_dir()` just like the CLI, so we chdir for
            // the duration of the call.
            let original = std::env::current_dir().expect("cwd");
            std::env::set_current_dir(&repo).expect("chdir");
            let result = git_sshripped_cli::__bench::cmd_unlock_default();
            std::env::set_current_dir(original).expect("restore cwd");
            assert!(result.is_ok(), "cmd_unlock failed: {result:?}");
        });
    });
}

criterion_group!(
    benches,
    bench_git_rev_parse,
    bench_sshripped_version,
    bench_unlock_process,
    bench_unlock_in_process,
);
criterion_main!(benches);
