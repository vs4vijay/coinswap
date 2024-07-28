#![cfg(feature = "integration-test")]
use std::process::Command;
use std::str;

#[tokio::test]
async fn test_cli_integration() {
    // Start the directory server
    let output = Command::new("cargo")
        .args(&["run", "--bin", "directoryd"])
        .output()
        .expect("Failed to start directory server");
    assert!(output.status.success());

    // Start the maker servers
    let maker_ports = vec![6102, 16102];
    for port in &maker_ports {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "makerd", "--", "--port", &port.to_string()])
            .output()
            .expect("Failed to start maker server");
        assert!(output.status.success());
    }

    // Start the taker client
    let output = Command::new("cargo")
        .args(&["run", "--bin", "taker-cli", "--", "--amount", "500000", "--makers", "2"])
        .output()
        .expect("Failed to start taker client");
    assert!(output.status.success());

    // Check the output of the taker client
    let stdout = str::from_utf8(&output.stdout).expect("Failed to read stdout");
    assert!(stdout.contains("Coinswap completed successfully"));

    // Stop the maker servers
    for port in &maker_ports {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "makerd", "--", "--port", &port.to_string(), "--stop"])
            .output()
            .expect("Failed to stop maker server");
        assert!(output.status.success());
    }

    // Stop the directory server
    let output = Command::new("cargo")
        .args(&["run", "--bin", "directoryd", "--", "--stop"])
        .output()
        .expect("Failed to stop directory server");
    assert!(output.status.success());
}
