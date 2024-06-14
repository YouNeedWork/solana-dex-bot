use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use validator::Validate;
#[instrument(name = "wallet")]
pub async fn wallet() -> &'static str {
    // user_id
    "wallet"
}

// create solana wallet
pub async fn create_wallet() {}

// remove solana wallet
pub async fn remove_wallet() {}

// set default wallet
pub async fn set_default_wallet() {}

// 定义一个结构体来表示你的 JSON 响应
#[derive(Serialize)]
struct ResponseData<T> {
    message: String,
    data: T,
    code: u32,
}
fn generator_response<T>(msg: String, code: u32, data: T) -> Json<ResponseData<T>> {
    Json(ResponseData {
        message: msg,
        code,
        data,
    })
}

#[derive(Debug, Validate, Deserialize)]
pub struct ImportWalletParams {
    #[validate(length(equal = 87))]
    pub private_key: String,
}

// pub async fn import_wallet(Json(frm): Json<ImportWalletParams>) -> Json<ResponseData>{
//     // frm.private_key.len()
//     if frm.private_key.len() != 87 {
//             let msg = "Your private key is in the wrong format".parse().unwrap();
//         return  generator_response(msg,401,[])
//     }
//     generator_response("".parse().unwrap(),401,[])
// }

pub struct SetWalletName {
    pub name: String,
}
pub async fn set_wallet_name() {}
