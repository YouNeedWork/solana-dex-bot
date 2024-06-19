-- Your SQL goes here
CREATE TABLE "wallets" (
  "id" serial PRIMARY KEY,
  "private_key" varchar NOT NULL,
  "wallet_address" varchar NOT NULL,
  "user_id" BIGINT NOT NULL,
  "tip" BIGINT NOT NULL DEFAULT 5000,
  "slippage" BIGINT NOT NULL DEFAULT 80,
  "is_default" boolean NOT NULL DEFAULT false,
  "create_at" timestamp,
  "update_at" timestamp
);

