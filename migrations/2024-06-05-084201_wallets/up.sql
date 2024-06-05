-- Your SQL goes here
CREATE TABLE "wallets" (
  "id" serial PRIMARY KEY,
  "private_key" varchar(200),
  "wallet_address" varchar(200),
  "user_id" int,
  "is_default" boolean,
  "create_at" timestamp,
  "update_at" timestamp
);

