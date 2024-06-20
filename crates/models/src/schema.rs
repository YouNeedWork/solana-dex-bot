// @generated automatically by Diesel CLI.

diesel::table! {
    history (id) {
        id -> Int4,
        wallet_id -> Int4,
        token_a -> Varchar,
        token_b -> Varchar,
        lp -> Varchar,
        amount -> Varchar,
        price -> Varchar,
        create_at -> Nullable<Timestamp>,
        update_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    hold_coins (id) {
        id -> Int4,
        wallet_id -> Int4,
        token_a -> Varchar,
        token_b -> Varchar,
        lp -> Varchar,
        amount -> Varchar,
        avg_price -> Varchar,
        create_at -> Nullable<Timestamp>,
        update_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    wallets (id) {
        id -> Int4,
        private_key -> Varchar,
        wallet_address -> Varchar,
        user_id -> Int8,
        tip -> Int8,
        slippage -> Int8,
        is_default -> Bool,
        create_at -> Nullable<Timestamp>,
        update_at -> Nullable<Timestamp>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(history, hold_coins, wallets,);
