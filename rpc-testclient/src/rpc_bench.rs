
use std::{str::FromStr, path::{Path, PathBuf}, thread};
use std::cmp::{max, min};
use std::ops::{Add, Range};
use std::ptr::null;
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde_json::json;
use serde_json::Value::Null;

use solana_client::rpc_client::{RpcClient, RpcClientConfig};
use solana_client::rpc_request::RpcRequest::Custom;
use solana_sdk::{pubkey::Pubkey, commitment_config::{CommitmentConfig, CommitmentLevel}, signature::Keypair, signer::Signer, transaction::Transaction};

pub fn run_bench() {
    println!("BENCHMARK with std::sync::Mutex ....");
    run_bench_lock(LockMode::STD_MUTEX);
    println!("============================");
    println!("BENCHMARK with tokio::sync::Mutex ....");
    run_bench_lock(LockMode::TOKIO_MUTEX);
    println!()

//    total counter diff 2268
//    total counter diff 5939
}

enum LockMode {
    STD_MUTEX,
    TOKIO_MUTEX
}

fn run_bench_lock(lock_mode: LockMode) {
    let url = "http://localhost:21212";

    let run_until = Instant::now().add(Duration::from_secs(10));

    let thread_hello = thread::spawn(move || {
        let rpc_client = RpcClient::new(url);
        let req_hello = Custom {method :"say_hello"};
        let mut start_counter = u64::MAX;
        let mut highest_counter = u64::MIN;
        loop {
            let response_counter: u64 = rpc_client.send(req_hello, json!(null)).unwrap();
            start_counter = min(start_counter, response_counter);
            highest_counter = max(highest_counter, response_counter);
            // println!("response_counter {:?}", response_counter);
            // println!("diff {:?}", response_counter - start_counter);
            thread::sleep(Duration::from_micros(50));
            if Instant::now() > run_until {
                break;
            }
        }
        println!("total counter diff {:?}", highest_counter - start_counter);
        // total counter diff 5951 (without lock)
        // total counter diff 2117 (with lock, std::sync::Mutex)
        // total counter diff 6220 (with lock, tokio-mutex)

    });
    let thread_mutexes = thread::spawn(move || {
        let rpc_client = RpcClient::new(url);

        loop {
            let req_call_with_lock =
                match lock_mode {
                    LockMode::STD_MUTEX => Custom {method :"call_with_stdlock"},
                    LockMode::TOKIO_MUTEX => Custom {method :"call_with_tokiolock"},
                };
            let response: String = rpc_client.send(req_call_with_lock, json!(null)).unwrap();
            // println!("response {:?}", response);
            thread::sleep(Duration::from_millis(50));
            if Instant::now() > run_until {
                break;
            }
        }
    });

    thread_hello.join().unwrap();
    thread_mutexes.join().unwrap();

}

fn call_say_hello(url: String, run_until: Arc<Instant>) {
    let rpc_client = RpcClient::new(url);
    let req_hello = Custom {method :"say_hello"};
    for i in 0..10 {
        let response: String = rpc_client.send(req_hello, json!(null)).unwrap();
        // println!("response {:?}", response);
        thread::sleep(Duration::from_millis(1));
        if Instant::now() > *run_until {
            return;
        }
    }


}

fn call_mutex(url: String, run_until: Arc<Instant>) {
    let rpc_client = RpcClient::new(url);

    for i in 0..10 {
        let req_call_with_lock = Custom {method :"call_with_lock"};
        let response: String = rpc_client.send(req_call_with_lock, json!(null)).unwrap();
        println!("response {:?}", response);
        thread::sleep(Duration::from_millis(250));
        if Instant::now() > *run_until {
            return;
        }
    }
}
