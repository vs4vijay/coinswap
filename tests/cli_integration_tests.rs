#![cfg(feature = "integration-test")]
use std::process::Command;
use std::str;

#[tokio::test]
async fn test_directory_server_start() {
    // Start the directory server
    let mut output = Command::new("cargo")
        .args(&["run", "--bin", "directoryd"])
        .spawn()
        .expect("Failed to start directory server");

    // Check if the directory server has started
    // Wait for the process to complete
    let status = output.wait().expect("Failed to wait for directory server");

    // Assert that the process exited successfully
    assert!(status.success());
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

}

// #[tokio::test]
// async fn test_taker_client() {
//     // Start the directory server
//     let output = Command::new("cargo")
//         .args(&["run", "--bin", "directoryd"])
//         .output()
//         .expect("Failed to start directory server");
//     assert!(output.status.success());

//     // Start a single maker server
//     let maker_port = 6102;
//     let output = Command::new("cargo")
//         .args(&[
//             "run",
//             "--bin",
//             "makerd",
//             "--",
//             "--port",
//             &maker_port.to_string(),
//         ])
//         .output()
//         .expect("Failed to start maker server");
//     assert!(output.status.success());

//     // Start the taker client with different parameters
//     let output = Command::new("cargo")
//         .args(&[
//             "run",
//             "--bin",
//             "taker",
//             "--",
//             "--amount",
//             "1000000",
//             "--makers",
//             "1",
//         ])
//         .output()
//         .expect("Failed to start taker client");
//     assert!(output.status.success());

//     // Check the output of the taker client
//     let stdout = str::from_utf8(&output.stdout).expect("Failed to read stdout");
//     assert!(stdout.contains("Coinswap completed successfully"));
// }
