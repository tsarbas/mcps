use std::{fs, process::Command};

fn run(config: &str, command: &str, pattern: &str) -> (std::process::Output, String) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("config.toml");
    fs::write(&path, config).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_mcps"))
        .arg("--config")
        .arg(&path)
        .args([command, pattern])
        .output()
        .unwrap();
    (output, fs::read_to_string(path).unwrap())
}

#[test]
fn wildcard_updates_are_sorted_and_preserve_other_content() {
    let config = "# Keep this comment\nother = 42\n[mcp_servers.cloud_z]\nenabled = true\n[mcp_servers.local]\nenabled = true\n[mcp_servers.cloud_a]\ncommand = 'tool'\n";
    let (output, saved) = run(config, "disable", "cloud*");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "cloud_a disabled\ncloud_z disabled\n"
    );
    assert_eq!(
        saved,
        config.replace(
            "[mcp_servers.cloud_z]\nenabled = true",
            "[mcp_servers.cloud_z]\nenabled = false"
        ) + "enabled = false\n"
    );

    let (output, saved) = run(&saved, "enable", "*");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "cloud_a enabled\ncloud_z enabled\nlocal enabled\n"
    );
    let doc: toml_edit::DocumentMut = saved.parse().unwrap();
    for name in ["cloud_a", "cloud_z", "local"] {
        assert_eq!(doc["mcp_servers"][name]["enabled"].as_bool(), Some(true));
    }
}

#[test]
fn errors_leave_file_unchanged_and_print_no_success() {
    for (config, pattern, error) in [
        (
            "[mcp_servers.a]\nenabled = true\n",
            "missing*",
            "No MCP servers match",
        ),
        ("[mcp_servers.a]\nenabled = true\n", "missing", "not found"),
        (
            "[mcp_servers.a]\nenabled = true\n[mcp_servers.z]\nenabled = 'bad'\n",
            "*",
            "non-boolean",
        ),
        (
            "[mcp_servers]\na = {}\nz = 42\n",
            "*",
            "invalid table format",
        ),
        ("other = 42\n", "*", "no [mcp_servers] section"),
    ] {
        let (output, saved) = run(config, "disable", pattern);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8(output.stderr).unwrap().contains(error));
        assert_eq!(saved, config);
    }
}

#[test]
fn invalid_unmatched_server_does_not_block_update() {
    let (output, saved) = run(
        "[mcp_servers.good]\nenabled = false\n[mcp_servers.bad]\nenabled = 'bad'\n",
        "enable",
        "good*",
    );
    assert!(output.status.success());
    assert!(saved.contains("enabled = true"));
    assert!(saved.contains("enabled = 'bad'"));
}
