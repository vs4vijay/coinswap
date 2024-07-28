#![cfg(feature = "integration-test")]
use std::process::Command;
use std::str;

#[tokio::test]
async fn test_directory_server_start_stop() {
    // Start the directory server
    let output = Command::new("cargo")
        .args(&["run", "--bin", "directoryd"])
        .output()
        .expect("Failed to start directory server");
    assert!(output.status.success());

    // Stop the directory server
    let output = Command::new("cargo")
        .args(&["run", "--bin", "directoryd", "--", "--stop"])
        .output()
        .expect("Failed to stop directory server");
    assert!(output.status.success());
}

#[tokio::test]
async fn test_maker_server_start_stop() {
    // Start a single maker server
    let maker_port = 6102;
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "makerd",
            "--",
            "--port",
            &maker_port.to_string(),
        ])
        .output()
        .expect("Failed to start maker server");
    assert!(output.status.success());

    // Stop the maker server
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "makerd",
            "--",
            "--port",
            &maker_port.to_string(),
            "--stop",
        ])
        .output()
        .expect("Failed to stop maker server");
    assert!(output.status.success());
}

#[tokio::test]
async fn test_taker_client() {
    // Start the directory server
    let output = Command::new("cargo")
        .args(&["run", "--bin", "directoryd"])
        .output()
        .expect("Failed to start directory server");
    assert!(output.status.success());

    // Start a single maker server
    let maker_port = 6102;
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "makerd",
            "--",
            "--port",
            &maker_port.to_string(),
        ])
        .output()
        .expect("Failed to start maker server");
    assert!(output.status.success());

    // Start the taker client with different parameters
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "taker",
            "--",
            "--amount",
            "1000000",
            "--makers",
            "1",
        ])
        .output()
        .expect("Failed to start taker server");
    assert!(output.status.success());

    // Check the output of the taker client
    let stdout = str::from_utf8(&output.stdout).expect("Failed to read stdout");
    assert!(stdout.contains("Coinswap completed successfully"));

    // Stop the maker server
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "makerd",
            "--",
            "--port",
            &maker_port.to_string(),
            "--stop",
        ])
        .output()
        .expect("Failed to stop maker server");
    assert!(output.status.success());

    // Stop the directory server
    let output = Command::new("cargo")
        .args(&["run", "--bin", "directoryd", "--", "--stop"])
        .output()
        .expect("Failed to stop directory server");
    assert!(output.status.success());
}
