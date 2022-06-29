CREATE TABLE "records" (
  "id" VARCHAR(64) PRIMARY KEY,
  "user_id" VARCHAR(64) NOT NULL,
  -- 余裕持って長めに確保する(1文字しか許さないのはアプリケーションの責任)
  "character" VARCHAR(8) NOT NULL,
  "figure" JSONB NOT NULL,
  "created_at" TIMESTAMPTZ NOT NULL,
  "stroke_count" INTEGER NOT NULL
);

CREATE INDEX "records_user_id_idx" ON "records" ("user_id");
CREATE INDEX "records_character_idx" ON "records" ("character");
CREATE INDEX "records_created_at_idx" ON "records" ("created_at");
CREATE INDEX "records_stroke_count_idx" ON "records" ("stroke_count");

CREATE TABLE "characters" (
  "id" VARCHAR(64) PRIMARY KEY,
  "user_id" VARCHAR(64) NOT NULL,
  "character" VARCHAR(8) NOT NULL,
  "stroke_count" INTEGER NOT NULL,
  "created_at" TIMESTAMPTZ NOT NULL,
  "updated_at" TIMESTAMPTZ NOT NULL
);

CREATE INDEX "characters_user_id_idx" ON "characters" ("user_id");
CREATE INDEX "characters_character_idx" ON "characters" ("character");
CREATE INDEX "characters_stroke_count_idx" ON "characters" ("stroke_count");
CREATE INDEX "characters_created_at_idx" ON "characters" ("created_at");
CREATE INDEX "characters_updated_at_idx" ON "characters" ("updated_at");
CREATE UNIQUE INDEX "characters_user_id_character_idx" ON "characters" ("user_id", "character");
