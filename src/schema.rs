// @generated automatically by Diesel CLI.

diesel::table! {
    hold_coins (id) {
        id -> Int4,
        wallet_id -> Nullable<Int4>,
        #[max_length = 50]
        token_a -> Nullable<Varchar>,
        #[max_length = 50]
        token_b -> Nullable<Varchar>,
        #[max_length = 50]
        price -> Nullable<Varchar>,
        create_at -> Nullable<Timestamp>,
        update_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    wallets (id) {
        id -> Int4,
        #[max_length = 200]
        private_key -> Nullable<Varchar>,
        #[max_length = 200]
        wallet_address -> Nullable<Varchar>,
        user_id -> Nullable<Int4>,
        is_default -> Nullable<Bool>,
        create_at -> Nullable<Timestamp>,
        update_at -> Nullable<Timestamp>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    hold_coins,
    wallets,
);
