CREATE TABLE "records" (
  "id" VARCHAR(64) PRIMARY KEY,
  "user_id" VARCHAR(64) NOT NULL,
  -- 余裕持って長めに確保する(1文字しか許さないのはアプリケーションの責任)
  "character" VARCHAR(8),
  "figure" JSONB NOT NULL,
  "created_at" TIMESTAMPTZ NOT NULL
);

CREATE INDEX "records_user_id_idx" ON "records" ("user_id");
CREATE INDEX "records_character_idx" ON "records" ("character");
CREATE INDEX "records_created_at_idx" ON "records" ("created_at");

