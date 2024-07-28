use std::env;
use std::process::Command;
use std::{convert::TryFrom, fs, path::PathBuf, str::FromStr};


use bitcoin::Amount;
use bitcoind::bitcoincore_rpc::{Auth, Client, RpcApi};
use coinswap::wallet::RPCConfig;
use coinswap::wallet::Wallet;

#[test]
fn test_send_to_address() {
    // Set up the environment
    let rpc_config = RPCConfig {
        url: "http://localhost:18443".to_string(),
        auth: Auth::UserPass("regtestrpcuser".to_string(), "regtestrpcpass".to_string()),
        network: bitcoin::Network::Regtest,
        wallet_name: "wallet_name".to_string(),
    };

    let client = Client::try_from(&rpc_config)?;

    // Create a new address for testing
    let address = client.get_new_address(None, None).expect("Failed to get new address");

    // Define the amount to send
    let amount = Amount::from_str("0.001").expect("Invalid amount");

    // Run the send_to_address command
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("taker-cli")
        .arg("send_to_address")
        .arg(format!("{}:{}", address, amount))
        .output()
        .expect("Failed to execute command");

    // Check the command output
    assert!(output.status.success(), "Command execution failed");

    // Verify the transaction
    let balance = client.get_balance(None, None).expect("Failed to get balance");
    assert_eq!(balance, amount, "Balance does not match the sent amount");
}
