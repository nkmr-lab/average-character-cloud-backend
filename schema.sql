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
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  "version" INTEGER NOT NULL,
  "ratio" INTEGER NOT NULL DEFAULT 100,
  "disabled" BOOLEAN NOT NULL DEFAULT FALSE,
  PRIMARY KEY ("user_id", "character", "stroke_count")
);

CREATE INDEX "character_configs_user_id_idx" ON "public"."character_configs" ("user_id");
CREATE INDEX "character_configs_character_idx" ON "public"."character_configs" ("character");
CREATE INDEX "character_configs_stroke_count_idx" ON "public"."character_configs" ("stroke_count");
CREATE INDEX "character_configs_updated_at_idx" ON "public"."character_configs" ("updated_at");
CREATE INDEX "character_configs_version_idx" ON "public"."character_configs" ("version");
CREATE INDEX "character_configs_disabled_idx" ON "public"."character_configs" ("disabled");

CREATE TABLE "public"."user_configs" (
  "user_id" VARCHAR(64) PRIMARY KEY,
  "allow_sharing_character_configs" BOOLEAN NOT NULL,
  "allow_sharing_figure_records" BOOLEAN NOT NULL,
  "random_level" INTEGER NOT NULL DEFAULT 50,
  "shared_proportion" INTEGER NOT NULL DEFAULT 50,
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

CREATE TABLE "public"."files" (
  "id" VARCHAR(64) PRIMARY KEY,
  "user_id" VARCHAR(64) NOT NULL,
  "key" VARCHAR(255) NOT NULL,
  "mime_type" VARCHAR(255) NOT NULL,
  "size" INTEGER NOT NULL,
  "verified" BOOLEAN NOT NULL,
  "created_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  "version" INTEGER NOT NULL
);
CREATE INDEX "files_user_id_idx" ON "public"."files" ("user_id");

CREATE TABLE "public"."generate_templates" (
  "id" VARCHAR(64) PRIMARY KEY,
  "user_id" VARCHAR(64) NOT NULL,
  "background_image_file_id" VARCHAR(64) NOT NULL,
  "font_color" INTEGER NOT NULL,
  "writing_mode" INTEGER NOT NULL,
  "margin_block_start" INTEGER NOT NULL,
  "margin_inline_start" INTEGER NOT NULL,
  "line_spacing" INTEGER NOT NULL,
  "letter_spacing" INTEGER NOT NULL,
  "font_size" INTEGER NOT NULL,
  "font_weight" INTEGER NOT NULL,
  "created_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  "disabled" BOOLEAN NOT NULL,
  "version" INTEGER NOT NULL
);

CREATE INDEX "generate_templates_user_id_idx" ON "public"."generate_templates" ("user_id");
CREATE INDEX "generate_templates_version_idx" ON "public"."generate_templates" ("version");
CREATE INDEX "generate_templates_disabled_idx" ON "public"."generate_templates" ("disabled");
CREATE INDEX "generate_templates_created_at_idx" ON "public"."generate_templates" ("created_at");
CREATE INDEX "generate_templates_updated_at_idx" ON "public"."generate_templates" ("updated_at");
