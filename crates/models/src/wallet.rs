use crate::schema::wallets;
use anyhow::{bail, Result};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = wallets)]
pub struct WalletQuery {
    pub id: i32,
    pub private_key: String,
    pub wallet_address: String,
    pub user_id: i64,
    pub tip: i64,
    pub slippage: i64,
    pub is_default: bool,
    pub create_at: Option<chrono::NaiveDateTime>,
    pub update_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = wallets)]
pub struct Wallet {
    pub private_key: String,
    pub wallet_address: String,
    pub user_id: i64,
    pub tip: i64,
    pub slippage: i64,
    pub is_default: bool,
    pub create_at: Option<chrono::NaiveDateTime>,
    pub update_at: Option<chrono::NaiveDateTime>,
}

impl Wallet {
    pub fn new(
        private_key: String,
        wallet_address: String,
        user_id: i64,
        is_default: bool,
    ) -> Self {
        Wallet {
            private_key,
            wallet_address,
            user_id,
            is_default,
            tip: 500000,
            slippage: 800,
            create_at: Some(chrono::Utc::now().naive_utc()),
            update_at: Some(chrono::Utc::now().naive_utc()),
        }
    }

    pub fn set_default(&mut self) {
        self.is_default = true;
    }

    pub fn create(&self, conn: &mut PgConnection) -> Result<(), diesel::result::Error> {
        diesel::insert_into(wallets::table)
            .values(self)
            .execute(conn)?;
        Ok(())
    }

    pub fn update(&self, conn: &mut PgConnection) -> Result<(), diesel::result::Error> {
        use crate::schema::wallets::dsl::*;
        diesel::update(wallets.filter(wallet_address.eq(&self.wallet_address)))
            .set((
                is_default.eq(self.is_default),
                update_at.eq(Some(chrono::Utc::now().naive_utc())),
            ))
            .execute(conn)?;
        Ok(())
    }

    pub fn fetch_default_with_id(
        conn: &mut PgConnection,
        fetch_user_id: i64,
    ) -> Result<WalletQuery> {
        use crate::schema::wallets::dsl::*;

        wallets
            .filter(user_id.eq(fetch_user_id))
            .filter(is_default.eq(true))
            .first::<WalletQuery>(conn)
            .map_err(Into::into)
    }

    pub fn fetch_default(
        conn: &mut PgConnection,
        fetch_user_id: i64,
    ) -> Result<Self, diesel::result::Error> {
        use crate::schema::wallets::dsl::*;

        let r = wallets
            .filter(user_id.eq(fetch_user_id))
            .filter(is_default.eq(true))
            .first::<WalletQuery>(conn);

        r.map(|w| Wallet {
            private_key: w.private_key,
            wallet_address: w.wallet_address,
            user_id: w.user_id,
            tip: w.tip,
            slippage: w.slippage,
            is_default: w.is_default,
            create_at: w.create_at,
            update_at: w.update_at,
        })
    }
}
