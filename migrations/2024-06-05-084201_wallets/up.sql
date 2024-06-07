-- Your SQL goes here
CREATE TABLE "wallets" (
  "id" serial PRIMARY KEY,
  "private_key" varchar NOT NULL,
  "wallet_address" varchar NOT NULL,
  "user_id" int,
  "is_default" boolean,
  "create_at" timestamp,
  "update_at" timestamp
);

