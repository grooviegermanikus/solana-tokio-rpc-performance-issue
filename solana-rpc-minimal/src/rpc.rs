use std::cell::Cell;
use std::sync::{Arc, Mutex};
use crossbeam_channel::Sender;
use jsonrpc_http_server::jsonrpc_core::Metadata;

#[derive(Clone)]
pub struct JsonRpcRequestProcessor {
    // bank_forks: Arc<RwLock<BankForks>>,
    // block_commitment_cache: Arc<RwLock<BlockCommitmentCache>>,
    // blockstore: Arc<Blockstore>,
    // config: JsonRpcConfig,
    // snapshot_config: Option<SnapshotConfig>,
    // #[allow(dead_code)]
    // validator_exit: Arc<RwLock<Exit>>,
    // health: Arc<RpcHealth>,
    // cluster_info: Arc<ClusterInfo>,
    // genesis_hash: Hash,
    // !!!NOTE!!!: std::sync::Mutex,
    // pub transaction_sender: Arc<Mutex<Cell<u32>>>,
    pub transaction_sender_tokio_mutex: Arc<tokio::sync::Mutex<Cell<u32>>>,
    pub transaction_sender_std_mutex: Arc<std::sync::Mutex<Cell<u32>>>,
    // pub transaction_sender: Arc<Mutex<Sender<TransactionInfo>>>,
    // bigtable_ledger_storage: Option<solana_storage_bigtable::LedgerStorage>,
    // optimistically_confirmed_bank: Arc<RwLock<OptimisticallyConfirmedBank>>,
    // largest_accounts_cache: Arc<RwLock<LargestAccountsCache>>,
    // max_slots: Arc<MaxSlots>,
    // leader_schedule_cache: Arc<LeaderScheduleCache>,
    // max_complete_transaction_status_slot: Arc<AtomicU64>,
    // prioritization_fee_cache: Arc<PrioritizationFeeCache>,
}

pub struct TransactionInfo {
    // pub signature: Signature,
    // pub wire_transaction: Vec<u8>,
    // pub last_valid_block_height: u64,
    // pub durable_nonce_info: Option<(Pubkey, Hash)>,
    // pub max_retries: Option<usize>,
    // retries: usize,
    // /// Last time the transaction was sent
    // last_sent_time: Option<Instant>,
}

impl JsonRpcRequestProcessor {
    pub fn new() -> Self {
        Self {
            transaction_sender_tokio_mutex: Arc::new(tokio::sync::Mutex::new(Cell::new(100))),
            transaction_sender_std_mutex: Arc::new(std::sync::Mutex::new(Cell::new(100)))
        }
    }
}

impl Metadata for JsonRpcRequestProcessor {}


use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

// #[rpc]
// pub trait Minimal {
//     type Metadata;
//
//     #[rpc(name = "getHealth")]
//     fn get_health(&self, meta: Self::Metadata) -> jsonrpc_core::Result<String>;
// }
//
// pub struct MinimalImpl;
// impl Minimal for MinimalImpl {
//     type Metadata = JsonRpcRequestProcessor;
//
//     fn get_health(&self, meta: Self::Metadata) -> jsonrpc_core::Result<String> {
//
//         Ok(String::from("foobar_health"))
//     }
//
// }
