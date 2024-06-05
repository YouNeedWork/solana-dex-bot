-- Your SQL goes here
CREATE TABLE "hold_coins" (
  "id" serial PRIMARY KEY,
  "wallet_id" int,
  "token_a" varchar(50),
  "token_b" varchar(50),
  "price" varchar(50),
  "create_at" timestamp,
  "update_at" timestamp
);
