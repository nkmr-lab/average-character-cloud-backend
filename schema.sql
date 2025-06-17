CREATE TABLE "public"."figure_records" (
  "id" VARCHAR(64) PRIMARY KEY,
  "user_id" VARCHAR(64) NOT NULL,
  -- 余裕持って長めに確保する(1文字しか許さないのはアプリケーションの責任)
  "character" VARCHAR(8) NOT NULL,
  "figure" JSONB NOT NULL,
  "created_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  "stroke_count" INTEGER NOT NULL,
  "version" INTEGER NOT NULL DEFAULT 1,
  "disabled" BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX "figure_records_user_id_idx" ON "public"."figure_records" ("user_id");
CREATE INDEX "figure_records_character_idx" ON "public"."figure_records" ("character");
CREATE INDEX "figure_records_created_at_idx" ON "public"."figure_records" ("created_at");
CREATE INDEX "figure_records_stroke_count_idx" ON "public"."figure_records" ("stroke_count");
CREATE INDEX "figure_records_version_idx" ON "public"."figure_records" ("version");
CREATE INDEX "figure_records_disabled_idx" ON "public"."figure_records" ("disabled");

CREATE TABLE "public"."character_configs" (
  "user_id" VARCHAR(64) NOT NULL,
  "character" VARCHAR(8) NOT NULL,
  "stroke_count" INTEGER NOT NULL,
  "created_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  "version" INTEGER NOT NULL,
  "ratio" INTEGER NOT NULL DEFAULT 100,
  PRIMARY KEY ("user_id", "character", "stroke_count")
);

CREATE INDEX "character_configs_stroke_count_idx" ON "public"."character_configs" ("stroke_count");
CREATE INDEX "character_configs_created_at_idx" ON "public"."character_configs" ("created_at");
CREATE INDEX "character_configs_updated_at_idx" ON "public"."character_configs" ("updated_at");
CREATE INDEX "character_configs_version_idx" ON "public"."character_configs" ("version");

CREATE TABLE "public"."user_configs" (
  "user_id" VARCHAR(64) PRIMARY KEY,
  "allow_sharing_character_configs" BOOLEAN NOT NULL,
  "allow_sharing_figure_records" BOOLEAN NOT NULL,
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  "version" INTEGER NOT NULL
);

CREATE INDEX "user_configs_updated_at_idx" ON "public"."user_configs" ("updated_at");
CREATE INDEX "user_configs_version_idx" ON "public"."user_configs" ("version");

CREATE TABLE "public"."character_config_seeds" (
  "character" VARCHAR(8) NOT NULL,
  "stroke_count" INTEGER NOT NULL,
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  "ratio" INTEGER NOT NULL DEFAULT 100,
  PRIMARY KEY ("character", "stroke_count")
);

CREATE INDEX "character_config_seeds_updated_at_idx" ON "public"."character_config_seeds" ("updated_at");
