CREATE TABLE "figure_records" (
  "id" VARCHAR(64) PRIMARY KEY,
  "user_id" VARCHAR(64) NOT NULL,
  -- 余裕持って長めに確保する(1文字しか許さないのはアプリケーションの責任)
  "character" VARCHAR(8) NOT NULL,
  "figure" JSONB NOT NULL,
  "created_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  "stroke_count" INTEGER NOT NULL
);

CREATE INDEX "figure_records_user_id_idx" ON "figure_records" ("user_id");
CREATE INDEX "figure_records_character_idx" ON "figure_records" ("character");
CREATE INDEX "figure_records_created_at_idx" ON "figure_records" ("created_at");
CREATE INDEX "figure_records_stroke_count_idx" ON "figure_records" ("stroke_count");

CREATE TABLE "character_configs" (
  "id" VARCHAR(64) PRIMARY KEY,
  "user_id" VARCHAR(64) NOT NULL,
  "character" VARCHAR(8) NOT NULL,
  "stroke_count" INTEGER NOT NULL,
  "created_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  "version" INTEGER NOT NULL
);

CREATE INDEX "character_configs_user_id_idx" ON "character_configs" ("user_id");
CREATE INDEX "character_configs_character_idx" ON "character_configs" ("character");
CREATE INDEX "character_configs_stroke_count_idx" ON "character_configs" ("stroke_count");
CREATE INDEX "character_configs_created_at_idx" ON "character_configs" ("created_at");
CREATE INDEX "character_configs_updated_at_idx" ON "character_configs" ("updated_at");
CREATE INDEX "character_configs_version_idx" ON "character_configs" ("version");
CREATE UNIQUE INDEX "character_configs_user_id_character_idx" ON "character_configs" ("user_id", "character");

CREATE TABLE "user_configs" (
  "user_id" VARCHAR(64) PRIMARY KEY,
  "allow_sharing_character_configs" BOOLEAN NOT NULL,
  "allow_sharing_figure_records" BOOLEAN NOT NULL,
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  "version" INTEGER NOT NULL
);
