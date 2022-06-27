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
