use solana_program::pubkey::Pubkey;

#[derive(Clone)]
pub struct User {
    pub id: i64,
    pub uid: i64,
    pub username: String,
    pub wallet_address: Pubkey,
    pub private_key: String,
    pub tip: i64,
    pub slippage: i64,
}
