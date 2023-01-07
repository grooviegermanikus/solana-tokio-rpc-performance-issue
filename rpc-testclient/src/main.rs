mod rpc_bench;

use std::{str::FromStr, path::{Path, PathBuf}};
use std::ptr::null;
use serde_json::json;
use serde_json::Value::Null;

use solana_client::rpc_client::{RpcClient, RpcClientConfig};
use solana_client::rpc_request::RpcRequest::Custom;
use solana_sdk::{pubkey::Pubkey, commitment_config::{CommitmentConfig, CommitmentLevel}, signature::Keypair, signer::Signer, transaction::Transaction};


fn main() {

    rpc_bench::run_bench();

}

