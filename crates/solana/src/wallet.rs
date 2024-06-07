use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signer::{keypair::Keypair, Signer};

pub struct Solana {
    keypair: Keypair,
    rpc: RpcClient,
}

impl Solana {
    pub fn genrate() -> Self {
        Self {
            keypair: Keypair::new(),
            rpc: RpcClient::new("https://alien-winter-orb.solana-mainnet.quiknode.pro/9c31f4035d451695084d9d94948726ea43683107/".to_string()),
        }
    }

    pub fn from_base58(s: &str) -> Self {
        Self {
            keypair: Keypair::from_base58_string(s),
            rpc: RpcClient::new("https://alien-winter-orb.solana-mainnet.quiknode.pro/9c31f4035d451695084d9d94948726ea43683107/".to_string()),
        }
    }

    pub fn private_key_base58(&self) -> String {
        self.keypair.to_base58_string()
    }

    pub async fn balance(&self) -> Result<u64> {
        self.rpc
            .get_balance(&self.keypair.pubkey())
            .await
            .map_err(|e| anyhow::format_err!(e))
    }
}
