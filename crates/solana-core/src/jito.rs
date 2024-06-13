use anyhow::Result;
use jito_protos::auth::{auth_service_client::AuthServiceClient, Role};
use jito_protos::searcher::searcher_service_client::SearcherServiceClient;
use jito_protos::searcher::{NextScheduledLeaderRequest, SubscribeBundleResultsRequest};
use jito_searcher_client::send_bundle_with_confirmation;
use jito_searcher_client::token_authenticator::ClientInterceptor;
use jito_searcher_client::BlockEngineConnectionResult;
use std::str::FromStr;
use std::time::Duration;
use tonic::transport::Endpoint;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::system_instruction::transfer;
use solana_sdk::transaction::Transaction;
use solana_sdk::{instruction::Instruction, transaction::VersionedTransaction};
use std::sync::Arc;
use tonic::{codegen::InterceptedService, transport::Channel};
use tracing::{error, info};

pub type SearcherClient = SearcherServiceClient<InterceptedService<Channel, ClientInterceptor>>;

pub async fn get_searcher_client(
    block_engine_url: &str,
    auth_keypair: &Arc<Keypair>,
) -> Result<SearcherClient> {
    let auth_channel = create_grpc_channel(block_engine_url).await?;
    let client_interceptor = ClientInterceptor::new(
        AuthServiceClient::new(auth_channel),
        auth_keypair,
        Role::Searcher,
    )
    .await?;

    let searcher_channel = create_grpc_channel(block_engine_url).await?;
    let searcher_client =
        SearcherServiceClient::with_interceptor(searcher_channel, client_interceptor);

    Ok(searcher_client)
}

pub async fn create_grpc_channel(url: &str) -> BlockEngineConnectionResult<Channel> {
    let mut endpoint = Endpoint::from_shared(url.to_string()).expect("invalid url");
    if url.starts_with("https") {
        endpoint = endpoint.tls_config(tonic::transport::ClientTlsConfig::new())?;
    }
    Ok(endpoint.connect().await?)
}

pub async fn wait_leader(
    searcher_client: &mut SearcherClient,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut is_leader_slot = false;
    while !is_leader_slot {
        let next_leader = searcher_client
            .get_next_scheduled_leader(NextScheduledLeaderRequest { regions: vec![] })
            .await
            .expect("gets next scheduled leader")
            .into_inner();
        let num_slots = next_leader.next_leader_slot - next_leader.current_slot;
        // give three slots for calc and bundle creation
        is_leader_slot = num_slots <= 3;
        info!(
            "next jito leader slot in {num_slots} slots in {}",
            next_leader.next_leader_region
        );
        if num_slots > 50 {
            error!("next leader slot too far in the future");
            return Ok(false);
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    Ok(true)
}

pub async fn send_swap_tx(
    ixs: &mut Vec<Instruction>,
    tip: u64,
    payer: &Keypair,
    searcher_client: &mut SearcherClient,
    rpc_client: &RpcClient,
) -> Result<()> {
    let mut bundle_results_subscription = searcher_client
        .subscribe_bundle_results(SubscribeBundleResultsRequest {})
        .await
        .expect("subscribe to bundle results")
        .into_inner();
    // build + sign the transactions
    let blockhash = rpc_client
        .get_latest_blockhash()
        .await
        .expect("get blockhash");

    // push tip ix
    ixs.push(transfer(
        &payer.pubkey(),
        &Pubkey::from_str("Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY")?,
        tip,
    ));

    let swap_tx = VersionedTransaction::from(Transaction::new_signed_with_payer(
        ixs.as_slice(),
        Some(&payer.pubkey()),
        &[payer],
        blockhash,
    ));

    send_bundle_with_confirmation(
        &[swap_tx],
        rpc_client,
        searcher_client,
        &mut bundle_results_subscription,
    )
    .await
    .map_err(|e| anyhow::format_err!("{:?}", e))
}
