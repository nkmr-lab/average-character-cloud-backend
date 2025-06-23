ALTER TABLE "public"."character_configs" ADD COLUMN "ratio" integer NOT NULL DEFAULT 100;
ALTER TABLE "public"."character_configs" ADD COLUMN "disabled" boolean NOT NULL DEFAULT false;
ALTER TABLE "public"."character_configs" DROP CONSTRAINT "character_configs_pkey";
ALTER TABLE "public"."character_configs" ADD PRIMARY KEY ("user_id", "character", "stroke_count");
ALTER TABLE "public"."user_configs" ADD COLUMN "random_level" integer NOT NULL DEFAULT 50;
ALTER TABLE "public"."user_configs" ADD COLUMN "shared_proportion" integer NOT NULL DEFAULT 50;
ALTER TABLE "public"."character_config_seeds" ADD COLUMN "ratio" integer NOT NULL DEFAULT 100;
ALTER TABLE "public"."character_config_seeds" DROP CONSTRAINT "character_config_seeds_pkey";
ALTER TABLE "public"."character_config_seeds" ADD PRIMARY KEY ("character", "stroke_count");
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
CREATE INDEX "character_configs_user_id_idx" ON "public"."character_configs" ("user_id");
CREATE INDEX "character_configs_character_idx" ON "public"."character_configs" ("character");
CREATE INDEX "character_configs_disabled_idx" ON "public"."character_configs" ("disabled");
CREATE INDEX "files_user_id_idx" ON "public"."files" ("user_id");
CREATE INDEX "generate_templates_user_id_idx" ON "public"."generate_templates" ("user_id");
CREATE INDEX "generate_templates_version_idx" ON "public"."generate_templates" ("version");
CREATE INDEX "generate_templates_disabled_idx" ON "public"."generate_templates" ("disabled");
CREATE INDEX "generate_templates_created_at_idx" ON "public"."generate_templates" ("created_at");
CREATE INDEX "generate_templates_updated_at_idx" ON "public"."generate_templates" ("updated_at");
DROP INDEX "public"."character_configs_created_at_idx";
ALTER TABLE "public"."character_configs" DROP COLUMN "created_at";
