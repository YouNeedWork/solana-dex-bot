--- Your SQL goes here
CREATE TABLE "history" (
  "id" serial PRIMARY KEY,
  "wallet_id" int NOT NULL,
  "token_a" varchar NOT NULL,
  "token_b" varchar NOT NULL,
  "lp" varchar NOT NULL,
  "amount" varchar NOT NULL,
  "price" varchar NOT NULL,
  "create_at" timestamp,
  "update_at" timestamp
);
