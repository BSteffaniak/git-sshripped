use std::fs;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

const TEST_PRIVATE_KEY: &str = "-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW
QyNTUxOQAAACBPLYMx485X5KACIiNYV5UD5QQwCFryTNF8UmGGuGziowAAAJjwIMUg8CDF
IAAAAAtzc2gtZWQyNTUxOQAAACBPLYMx485X5KACIiNYV5UD5QQwCFryTNF8UmGGuGziow
AAAEBh012cG+6OBMUHrxxxVQQ73Y32TrNRJcpZdI11XEJ8EE8tgzHjzlfkoAIiI1hXlQPl
BDAIWvJM0XxSYYa4bOKjAAAAEnRlc3RAZ2l0LXNzaC1jcnlwdAECAw==
-----END OPENSSH PRIVATE KEY-----
";

const TEST_PUBLIC_KEY: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIE8tgzHjzlfkoAIiI1hXlQPlBDAIWvJM0XxSYYa4bOKj test@git-ssh-crypt\n";

fn run_ok(cmd: &mut Command) -> Vec<u8> {
    let output = cmd.output().expect("command execution should succeed");
    if !output.status.success() {
        panic!(
            "command failed: status={}\nstdout={}\nstderr={}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    output.stdout
}

fn configure_filter_paths(repo: &Path, bin: &str) {
    run_ok(Command::new("git").current_dir(repo).args([
        "config",
        "--local",
        "filter.git-ssh-crypt.process",
        &format!("{bin} filter-process"),
    ]));
    run_ok(Command::new("git").current_dir(repo).args([
        "config",
        "--local",
        "filter.git-ssh-crypt.clean",
        &format!("{bin} clean --path %f"),
    ]));
    run_ok(Command::new("git").current_dir(repo).args([
        "config",
        "--local",
        "filter.git-ssh-crypt.smudge",
        &format!("{bin} smudge --path %f"),
    ]));
}

#[test]
fn filter_process_roundtrip_with_lock_unlock() {
    let bin = env!("CARGO_BIN_EXE_git-ssh-crypt");
    let temp = TempDir::new().expect("temp dir should create");
    let repo = temp.path();

    run_ok(Command::new("git").current_dir(repo).args(["init"]));
    run_ok(
        Command::new("git")
            .current_dir(repo)
            .args(["config", "user.name", "test"]),
    );
    run_ok(Command::new("git").current_dir(repo).args([
        "config",
        "user.email",
        "test@example.com",
    ]));

    let keys_dir = repo.join("keys");
    fs::create_dir_all(&keys_dir).expect("keys dir should create");
    let private_key = keys_dir.join("id_ed25519");
    let public_key = keys_dir.join("id_ed25519.pub");
    fs::write(&private_key, TEST_PRIVATE_KEY).expect("private key should write");
    fs::write(&public_key, TEST_PUBLIC_KEY).expect("public key should write");

    run_ok(Command::new(bin).current_dir(repo).args([
        "init",
        "--pattern",
        "secrets/**",
        "--recipient-key",
        public_key.to_str().expect("public key path should be utf8"),
    ]));

    configure_filter_paths(repo, bin);

    run_ok(
        Command::new(bin).current_dir(repo).args([
            "unlock",
            "--identity",
            private_key
                .to_str()
                .expect("private key path should be utf8"),
        ]),
    );

    let secret_dir = repo.join("secrets");
    fs::create_dir_all(&secret_dir).expect("secrets dir should create");
    let secret_file = secret_dir.join("app.env");
    fs::write(&secret_file, b"API_KEY=top_secret\n").expect("secret file should write");

    run_ok(Command::new("git").current_dir(repo).args(["add", "."]));
    run_ok(
        Command::new("git")
            .current_dir(repo)
            .args(["commit", "-m", "test commit"]),
    );

    let staged_blob = run_ok(
        Command::new("git")
            .current_dir(repo)
            .args(["show", "HEAD:secrets/app.env"]),
    );
    assert!(staged_blob.starts_with(b"GSC1"));
    assert!(
        !staged_blob
            .windows(b"top_secret".len())
            .any(|w| w == b"top_secret")
    );

    run_ok(Command::new(bin).current_dir(repo).args(["lock"]));
    fs::remove_file(&secret_file).expect("secret file should be removable while locked");
    run_ok(
        Command::new("git")
            .current_dir(repo)
            .args(["checkout", "--", "secrets/app.env"]),
    );
    let locked_view = fs::read(&secret_file).expect("locked view should read");
    assert!(locked_view.starts_with(b"GSC1"));

    run_ok(
        Command::new(bin).current_dir(repo).args([
            "unlock",
            "--identity",
            private_key
                .to_str()
                .expect("private key path should be utf8"),
        ]),
    );
    fs::remove_file(&secret_file).expect("secret file should be removable while unlocked");
    run_ok(
        Command::new("git")
            .current_dir(repo)
            .args(["checkout", "--", "secrets/app.env"]),
    );
    let unlocked_view = fs::read_to_string(&secret_file).expect("unlocked view should read utf8");
    assert_eq!(unlocked_view, "API_KEY=top_secret\n");
}
