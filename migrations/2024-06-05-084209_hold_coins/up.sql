-- Your SQL goes here
CREATE TABLE "hold_coins" (
  "id" serial PRIMARY KEY,
  "wallet_id" int NOT NULL,
  "token_a" varchar NOT NULL,
  "token_b" varchar NOT NULL,
  "lp" varchar NOT NULL,
  "amount" varchar NOT NULL,
  "avg_price" varchar NOT NULL,
  "create_at" timestamp,
  "update_at" timestamp
);

CREATE UNIQUE INDEX idx_wallet_token ON hold_coins (wallet_id,token_b);
