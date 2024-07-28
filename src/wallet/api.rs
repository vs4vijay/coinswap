use std::{convert::TryFrom, fs, path::PathBuf, str::FromStr};

use std::collections::{HashMap, HashSet};

use bitcoin::{
    bip32::{ChildNumber, DerivationPath, Xpriv, Xpub},
    hashes::{hash160::Hash as Hash160, hex::FromHex},
    secp256k1,
    secp256k1::{Secp256k1, SecretKey},
    sighash::{EcdsaSighashType, SighashCache},
    Address, Amount, OutPoint, PublicKey, Script, ScriptBuf, Transaction, Txid,
};

use bdk::blockchain::{Blockchain, ConfigurableBlockchain, ElectrumBlockchainConfig};
use bdk::database::MemoryDatabase;
use bdk::wallet::{AddressIndex, Wallet as BdkWallet};
use bdk::{SyncOptions, FeeRate};

use crate::{
    protocol::contract,
    utill::{
        compute_checksum, generate_keypair, get_hd_path_from_descriptor,
        redeemscript_to_scriptpubkey,
    },
};

use super::{
    error::WalletError,
    storage::WalletStore,
    swapcoin::{IncomingSwapCoin, OutgoingSwapCoin, SwapCoin, WalletSwapCoin},
};

const HARDENDED_DERIVATION: &str = "m/84'/1'/0'";

/// Represents a Bitcoin wallet with associated functionality and data.
pub struct Wallet {
    pub(crate) bdk_wallet: BdkWallet<MemoryDatabase>,
    pub(crate) blockchain: Box<dyn Blockchain>,
    wallet_file_path: PathBuf,
    pub(crate) store: WalletStore,
}

/// Speicfy the keychain derivation path from [`HARDENDED_DERIVATION`]
/// Each kind represents an unhardened index value. Starting with External = 0.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum KeychainKind {
    External = 0isize,
    Internal,
}

impl KeychainKind {
    fn index_num(&self) -> u32 {
        match self {
            Self::External => 0,
            Self::Internal => 1,
        }
    }
}

const WATCH_ONLY_SWAPCOIN_LABEL: &str = "watchonly_swapcoin_label";

/// Enum representing different types of addresses to display.
#[derive(Clone, PartialEq, Debug)]
pub enum DisplayAddressType {
    /// Display all types of addresses.
    All,
    /// Display information related to the master key.
    MasterKey,
    /// Display addresses derived from the seed.
    Seed,
    /// Display information related to incoming swap transactions.
    IncomingSwap,
    /// Display information related to outgoing swap transactions.
    OutgoingSwap,
    /// Display information related to swap transactions (both incoming and outgoing).
    Swap,
    /// Display information related to incoming contract transactions.
    IncomingContract,
    /// Display information related to outgoing contract transactions.
    OutgoingContract,
    /// Display information related to contract transactions (both incoming and outgoing).
    Contract,
    /// Display information related to fidelity bonds.
    FidelityBond,
}

impl FromStr for DisplayAddressType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "all" => DisplayAddressType::All,
            "masterkey" => DisplayAddressType::MasterKey,
            "seed" => DisplayAddressType::Seed,
            "incomingswap" => DisplayAddressType::IncomingSwap,
            "outgoingswap" => DisplayAddressType::OutgoingSwap,
            "swap" => DisplayAddressType::Swap,
            "incomingcontract" => DisplayAddressType::IncomingContract,
            "outgoingcontract" => DisplayAddressType::OutgoingContract,
            "contract" => DisplayAddressType::Contract,
            "fidelitybond" => DisplayAddressType::FidelityBond,
            _ => Err("unknown type")?,
        })
    }
}

/// Enum representing additional data needed to spend a UTXO, in addition to `ListUnspentResultEntry`.
// data needed to find information  in addition to ListUnspentResultEntry
// about a UTXO required to spend it
#[derive(Debug, Clone)]
pub enum UTXOSpendInfo {
    SeedCoin {
        path: String,
        input_value: Amount,
    },
    SwapCoin {
        multisig_redeemscript: ScriptBuf,
    },
    TimelockContract {
        swapcoin_multisig_redeemscript: ScriptBuf,
        input_value: Amount,
    },
    HashlockContract {
        swapcoin_multisig_redeemscript: ScriptBuf,
        input_value: Amount,
    },
    FidelityBondCoin {
        index: u32,
        input_value: Amount,
    },
}

// Custom type to handle complex return values.
type SwapCoinsInfo<'a> = (
    Vec<(&'a IncomingSwapCoin, ListUnspentResultEntry)>,
    Vec<(&'a OutgoingSwapCoin, ListUnspentResultEntry)>,
);

impl Wallet {
    /// Initialize the wallet at a given path.
    pub fn init(
        path: &PathBuf,
        electrum_url: &str,
        seedphrase: String,
        passphrase: String,
    ) -> Result<Self, WalletError> {
        // Xpriv Derivation from seedphrase
        let mnemonic = bip39::Mnemonic::parse(seedphrase.clone())?;
        let seed = mnemonic.to_seed(passphrase.clone());
        let master_key = Xpriv::new_master(Network::Testnet, &seed)?;

        // Initialise wallet
        let file_name = path
            .file_name()
            .expect("file name expected")
            .to_str()
            .expect("expected")
            .to_string();
        let wallet_birthday = 0; // Placeholder value, update as needed
        let store = WalletStore::init(
            file_name,
            path,
            Network::Testnet,
            master_key,
            Some(wallet_birthday),
        )?;

        let bdk_wallet = BdkWallet::new(
            "wpkh([d34db33f/84'/1'/0']tpubD6NzVbkrYhZ4Y5Y4G1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1Q1
