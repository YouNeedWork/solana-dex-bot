use crate::schema::hold_coins::{self, wallet_id};
use anyhow::Result;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = hold_coins)]
pub struct HoldCoinQuery {
    pub id: i32,
    pub wallet_id: i32,
    pub token_a: String,
    pub token_b: String,
    pub lp: String,
    pub amount: String,
    pub avg_price: String,
    pub create_at: Option<chrono::NaiveDateTime>,
    pub update_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = hold_coins)]
pub struct HoldCoin {
    pub wallet_id: i32,
    pub token_a: String,
    pub token_b: String,
    pub lp: String,
    pub amount: String,
    pub avg_price: String,
    pub create_at: Option<chrono::NaiveDateTime>,
    pub update_at: Option<chrono::NaiveDateTime>,
}

impl HoldCoin {
    pub fn new(
        w_id: i32,
        token_a: String,
        token_b: String,
        lp: String,
        amount: String,
        avg_price: String,
    ) -> Self {
        Self {
            wallet_id: w_id,
            token_a,
            token_b,
            lp,
            amount,
            avg_price,
            create_at: Some(chrono::Utc::now().naive_utc()),
            update_at: Some(chrono::Utc::now().naive_utc()),
        }
    }

    pub fn create(&self, conn: &mut PgConnection) -> Result<(), diesel::result::Error> {
        diesel::insert_into(hold_coins::table)
            .values(self)
            .execute(conn)?;
        Ok(())
    }

    pub fn create_or_update(conn: &mut PgConnection, new_record: &HoldCoin) -> Result<usize> {
        use crate::schema::hold_coins::dsl::*;

        diesel::insert_into(hold_coins)
            .values(new_record)
            .on_conflict((wallet_id, token_b))
            .do_update()
            .set(amount.eq(new_record.amount.clone()))
            .execute(conn)
            .map_err(Into::into)
    }

    pub fn fetch_all(conn: &mut PgConnection, fetch_wallet_id: i32) -> Result<Vec<HoldCoinQuery>> {
        use crate::schema::hold_coins::dsl::*;

        hold_coins
            .filter(wallet_id.eq(fetch_wallet_id))
            .get_results(conn)
            .map_err(Into::into)
    }
}
