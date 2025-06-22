ALTER TABLE "public"."files" ALTER COLUMN "version" DROP DEFAULT;
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
