mod rpc;
mod stuff;

use std::cmp::Ordering;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, AtomicI64, AtomicU32};
use std::sync::atomic::Ordering::Relaxed;
use std::thread;
use std::thread::Builder;
use std::time::Duration;
use hyper::{Body, Request};
use jsonrpc_core::serde_json::json;
use jsonrpc_http_server::{
    hyper, AccessControlAllowOrigin, CloseHandle, DomainsValidation, RequestMiddleware,
    RequestMiddlewareAction, ServerBuilder};
use jsonrpc_http_server::jsonrpc_core::MetaIoHandler;
use tokio::runtime::Runtime;
use rpc::JsonRpcRequestProcessor;
use jsonrpc_http_server::jsonrpc_core::{IoHandler, Value, Params};
use tokio::time::Instant;

#[tokio::main]
async fn main() {

    const rpc_threads: usize = 5;
    const rpc_niceness_adj: i8 = 0;
    const max_request_body_size: usize = 50 * (1 << 10); // 50kB

    let ip_addr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
    let rpc_addr = SocketAddr::new(
        ip_addr,
        21212
        // stuff::find_available_port_in_range(ip_addr, (10000, 65535)).unwrap(),
    );

    println!("RPC to listen on {:?}", rpc_addr);


    let concurrent_requests = Arc::new(AtomicI64::new(0));


    // from rpc service
    let runtime = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(rpc_threads)
            .on_thread_start(move || renice_this_thread(rpc_niceness_adj).unwrap())
            .thread_name("solRpcEl")
            .enable_all()
            .build()
            .expect("Runtime"),
    );

    let clonedrt = runtime.clone();
    println!("Starting BigTable upload service");
    let thread = Builder::new()
        .name("solBigTUpload".to_string())
        .spawn(move || {
            // run:
            clonedrt.block_on(async {
                std::thread::sleep(std::time::Duration::from_secs(3));
            })
            // Self::run(
            //     runtime,
            //     bigtable_ledger_storage,
            //     blockstore,
            //     block_commitment_cache,
            //     max_complete_transaction_status_slot,
            //     config,
            //     exit,
            // )
        })
        .unwrap();

    let request_processor = JsonRpcRequestProcessor::new();

    let counter_hdl = concurrent_requests.clone();
    let thread_hdl = Builder::new()
        .name("solJsonRpcSvc".to_string())
        .spawn(move || {
            println!("Starting thread solJsonRpcSvc");
            renice_this_thread(rpc_niceness_adj).unwrap();

            let mut io = MetaIoHandler::default();

            io.add_method("say_hello", move |_params: Params| {
                // println!("saying hello ...");
                let cnt = counter_hdl.clone();
                async move {
                    let old = cnt.fetch_add(1, Relaxed);
                    Ok(json!(old+1))
                }
            });

            // call with a lock (e.g. request_airdrop)
            io.add_method_with_meta("call_with_stdlock", move |params: Params, meta| {
                async move {
                    let start = Instant::now();

                    _send_transaction_std_lock(meta);

                    let locked_time = start.elapsed();
                    println!("locked for {:?}", locked_time);

                    Ok(Value::String("locked for x ms".to_owned()))
                }
            });

            io.add_method_with_meta("call_with_tokiolock", move |params: Params, meta| {
                async move {
                    let start = Instant::now();

                    _send_transaction_tokio_lock(meta);

                    let locked_time = start.elapsed();
                    println!("locked for {:?}", locked_time);

                    Ok(Value::String("locked for x ms".to_owned()))
                }
            });

            io.add_method("burn_some_cpu_cycles", |params: Params| async {
                match params {
                    Params::None => {}
                    Params::Array(_) => {}
                    Params::Map(map) => {
                        // let milliseconds: u64 = map.get("milliseconds").sarse().unwrap();
                        //
                        // println!("burn {:?}", milliseconds);


                    }
                }
                Ok(Value::String("ashes".to_owned()))
            });


            // io.extend_with(rpc::MinimalImpl.to_delegate());
            // if full_api {
            //     io.extend_with(rpc_bank::BankDataImpl.to_delegate());
            //     io.extend_with(rpc_accounts::AccountsDataImpl.to_delegate());
            //     io.extend_with(rpc_accounts_scan::AccountsScanImpl.to_delegate());
            //     io.extend_with(rpc_full::FullImpl.to_delegate());
            //     io.extend_with(rpc_deprecated_v1_7::DeprecatedV1_7Impl.to_delegate());
            //     io.extend_with(rpc_deprecated_v1_9::DeprecatedV1_9Impl.to_delegate());
            // }
            // if obsolete_v1_7_api {
            //     io.extend_with(rpc_obsolete_v1_7::ObsoleteV1_7Impl.to_delegate());
            // }

            let request_middleware = RpcRequestMiddleware::new();

            // let request_middleware = RpcRequestMiddleware::new(
            //     ledger_path,
            //     snapshot_config,
            //     bank_forks.clone(),
            //     health.clone(),
            // );
            let server = ServerBuilder::with_meta_extractor(
                io,
                move |_req: &hyper::Request<hyper::Body>| request_processor.clone(),
            )
                .event_loop_executor(runtime.handle().clone())
                .threads(1)
                .cors(DomainsValidation::AllowOnly(vec![
                    AccessControlAllowOrigin::Any,
                ]))
                .cors_max_age(86400)
                // handle non-rpc stuff
                .request_middleware(request_middleware)
                .max_request_body_size(max_request_body_size)
                .start_http(&rpc_addr);

            if let Err(e) = server {
                println!(
                        "JSON RPC service unavailable error: {:?}. \n\
                           Also, check that port {} is not already in use by another application",
                        e,
                        rpc_addr.port()
                    );
                // close_handle_sender.send(Err(e.to_string())).unwrap();
                return;
            }

            let server = server.unwrap();
            // close_handle_sender.send(Ok(server.close_handle())).unwrap();
            server.wait();
            // exit_bigtable_ledger_upload_service.store(true, Ordering::Relaxed);
        })
        .unwrap();

    let c2 = concurrent_requests.clone();
    let thread_counter = Builder::new()
        .spawn( move || {
            loop {
                println!("counter {:?}", c2);
                thread::sleep(Duration::from_millis(3000));
            }

        }).unwrap();


    thread_counter.join();
    thread_hdl.join();

}

fn _send_transaction_std_lock(
    meta: JsonRpcRequestProcessor
    // signature: Signature,
    // wire_transaction: Vec<u8>,
    // last_valid_block_height: u64,
    // durable_nonce_info: Option<(Pubkey, Hash)>,
    // max_retries: Option<usize>,
) {
    println!("send transaction (std mutex)");
    // let transaction_info = TransactionInfo::new(
    //     signature,
    //     wire_transaction,
    //     last_valid_block_height,
    //     durable_nonce_info,
    //     max_retries,
    //     None,
    // );

    let value = meta.transaction_sender_std_mutex
        .lock()
        .unwrap();

    thread::sleep(Duration::from_millis(200));

    let old = value.replace(42);


    // Ok(signature.to_string())
}

async fn _send_transaction_tokio_lock(
    meta: JsonRpcRequestProcessor
    // signature: Signature,
    // wire_transaction: Vec<u8>,
    // last_valid_block_height: u64,
    // durable_nonce_info: Option<(Pubkey, Hash)>,
    // max_retries: Option<usize>,
) {
    println!("send transaction (fixed)");
    // let transaction_info = TransactionInfo::new(
    //     signature,
    //     wire_transaction,
    //     last_valid_block_height,
    //     durable_nonce_info,
    //     max_retries,
    //     None,
    // );

    let value = meta.transaction_sender_tokio_mutex
        .lock();

    thread::sleep(Duration::from_millis(200));

    let old = value.await.replace(42);


    // Ok(signature.to_string())
}

async fn burn_some_cpu_cycles() {

}

// From perf/src/thread.rs - only useful on linux
fn renice_this_thread(adjustment: i8) -> Result<(), String> {
    println!("renice not implemented");
    Ok(())
}

struct RpcRequestMiddleware {

}

impl RpcRequestMiddleware {
    fn new() -> Self {
        Self {}
    }
}

impl RequestMiddleware for RpcRequestMiddleware {
    fn on_request(&self, request: Request<Body>) -> RequestMiddlewareAction {

        if request.uri().path() == "/" {
            // no logging
            request.into()
        } else if request.uri().path() == "/health" {
            hyper::Response::builder()
                .status(hyper::StatusCode::OK)
                .body(hyper::Body::from("health_check_notimplemented"))
                .unwrap()
                .into()
        } else {
            println!("request uri: {}", request.uri());
            request.into()
        }

    }
}

