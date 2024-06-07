-- Your SQL goes here
CREATE TABLE "hold_coins" (
  "id" serial PRIMARY KEY,
  "wallet_id" int NOT NULL,
  "token_a" varchar NOT NULL,
  "token_b" varchar NOT NULL,
  "price" varchar NOT NULL,
  "create_at" timestamp,
  "update_at" timestamp
);
