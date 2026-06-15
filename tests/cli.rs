use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn version_prints_binary_name_and_version() {
    Command::cargo_bin("sshx")
        .unwrap()
        .arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains("sshx "));
}

#[test]
fn dry_run_connect_builds_real_ssh_command() {
    let tempdir = tempdir().unwrap();
    let path = tempdir.path().join("config");
    std::fs::write(
        &path,
        "Host test-box\n    HostName example.com\n    Port 2222\n    User alice\n",
    )
    .unwrap();

    Command::cargo_bin("sshx")
        .unwrap()
        .args(["--config", path.to_str().unwrap(), "--dry-run", "test-box"])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "Command: ssh -p 2222 alice@example.com",
        ));
}

#[test]
fn dry_run_resolves_alias_without_executing() {
    let tempdir = tempdir().unwrap();
    let path = tempdir.path().join("config");
    std::fs::write(
        &path,
        "Host prod-app\n    HostName 203.0.113.50\n    User deploy\n    ## sshx: alias = prod\n",
    )
    .unwrap();

    Command::cargo_bin("sshx")
        .unwrap()
        .args(["--config", path.to_str().unwrap(), "--dry-run", "prod"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Command: ssh deploy@203.0.113.50"));
}

#[test]
fn dry_run_resolves_host_from_included_config() {
    let tempdir = tempdir().unwrap();
    let included = tempdir.path().join("included.conf");
    let root = tempdir.path().join("config");

    std::fs::write(
        &included,
        "Host included-box\n    HostName 10.10.10.10\n    User included\n",
    )
    .unwrap();
    std::fs::write(
        &root,
        format!(
            "Include {}\nHost root-box\n    HostName 192.0.2.10\n",
            included.display()
        ),
    )
    .unwrap();

    Command::cargo_bin("sshx")
        .unwrap()
        .args([
            "--config",
            root.to_str().unwrap(),
            "--dry-run",
            "included-box",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "Command: ssh included@10.10.10.10",
        ));
}
