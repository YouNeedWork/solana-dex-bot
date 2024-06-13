// @generated automatically by Diesel CLI.

diesel::table! {
    hold_coins (id) {
        id -> Int4,
        wallet_id -> Int4,
        token_a -> Varchar,
        token_b -> Varchar,
        price -> Varchar,
        create_at -> Nullable<Timestamp>,
        update_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    wallets (id) {
        id -> Int4,
        private_key -> Varchar,
        wallet_address -> Varchar,
        user_id -> Nullable<Int4>,
        is_default -> Nullable<Bool>,
        create_at -> Nullable<Timestamp>,
        update_at -> Nullable<Timestamp>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(hold_coins, wallets,);
