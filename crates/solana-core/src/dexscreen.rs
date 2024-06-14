use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairResult {
    pub schema_version: String,
    pub pairs: Vec<Pair>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pair {
    pub chain_id: String,
    pub dex_id: String,
    pub url: String,
    pub pair_address: String,
    pub base_token: BaseToken,
    pub quote_token: QuoteToken,
    pub price_native: String,
    pub price_usd: String,
    pub txns: Txns,
    pub volume: Volume,
    pub price_change: PriceChange,
    pub liquidity: Liquidity,
    pub fdv: i64,
    pub pair_created_at: i64,
    pub info: Info,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseToken {
    pub address: String,
    pub name: String,
    pub symbol: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteToken {
    pub address: String,
    pub name: String,
    pub symbol: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Txns {
    pub m5: M5,
    pub h1: H1,
    pub h6: H6,
    pub h24: H24,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct M5 {
    pub buys: i64,
    pub sells: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct H1 {
    pub buys: i64,
    pub sells: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct H6 {
    pub buys: i64,
    pub sells: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct H24 {
    pub buys: i64,
    pub sells: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Volume {
    pub h24: f64,
    pub h6: f64,
    pub h1: i64,
    pub m5: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceChange {
    pub m5: i64,
    pub h1: f64,
    pub h6: f64,
    pub h24: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Liquidity {
    pub usd: i64,
    pub base: i64,
    pub quote: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub image_url: String,
    pub websites: Vec<Website>,
    pub socials: Vec<Social>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Website {
    pub label: String,
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Social {
    #[serde(rename = "type")]
    pub type_field: String,
    pub url: String,
}

pub async fn search(token_address: &str) -> Result<PairResult> {
    let url = format!(
        "https://api.dexscreener.com/latest/dex/tokens/{}",
        token_address
    );

    let res = reqwest::get(&url).await?.json().await?;

    Ok(res)
}
