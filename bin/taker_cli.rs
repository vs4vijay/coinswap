use std::env;
use std::{convert::TryFrom, fs, path::PathBuf, str::FromStr};

use bitcoin::Amount;
use bitcoind::bitcoincore_rpc::{Auth, Client, RpcApi};
use coinswap::wallet::RPCConfig;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} send_to_address <address:amount>", args[0]);
        std::process::exit(1);
    }

    let command = &args[1];
    let address_amount = &args[2];

    if command == "send_to_address" {
        let (address, amount) = parse_address_amount(address_amount);
        send_to_address(address, amount);
    } else {
        eprintln!("Unknown command: {}", command);
        std::process::exit(1);
    }
}

fn parse_address_amount(address_amount: &str) -> (String, Amount) {
    let parts: Vec<&str> = address_amount.split(':').collect();
    if parts.len() != 2 {
        eprintln!("Invalid address:amount format");
        std::process::exit(1);
    }

    let address = parts[0].to_string();
    let amount = Amount::from_str(parts[1]).expect("Invalid amount");

    (address, amount)
}

fn send_to_address(address: String, amount: Amount) {
    let rpc_config = RPCConfig {
        url: "http://localhost:18443".to_string(),
        auth: Auth::UserPass("regtestrpcuser".to_string(), "regtestrpcpass".to_string()),
        network: bitcoin::Network::Regtest,
        wallet_name: "wallet_name".to_string(),
    };

    let client = Client::try_from(&rpc_config).expect("Failed to connect to Bitcoin RPC");

    client
        .send_to_address(
            &bitcoin::Address::from_str(&address).expect("Invalid address"),
            amount,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("Failed to send transaction");

    println!("Transaction sent to {} for amount {}", address, amount);
}
