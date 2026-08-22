use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn feeds_command(path: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rss-scout"))
        .args(["feeds", "--feeds"])
        .arg(path)
        .output()
        .expect("run rss-scout feeds")
}

#[test]
fn malformed_config_exits_nonzero() {
    let temp = tempfile::tempdir().expect("tempdir");
    let config = temp.path().join("malformed.toml");
    fs::write(&config, "[settings\nkeywords = 'rust'\n").expect("write malformed config");

    let output = feeds_command(&config);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("解析"));
}

#[test]
fn missing_config_exits_nonzero() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = feeds_command(&temp.path().join("missing.toml"));

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("无法读取"));
}

#[test]
fn canonical_tool_config_omits_unfiltered_show_hn_firehose() {
    let config = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("feeds-tools.toml");
    let output = feeds_command(&config);

    assert!(output.status.success());
    assert!(!String::from_utf8_lossy(&output.stdout).contains("ShowHN:all"));
}
