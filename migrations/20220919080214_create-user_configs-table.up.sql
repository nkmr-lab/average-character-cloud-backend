CREATE TABLE "user_configs" (
  "user_id" VARCHAR(64) PRIMARY KEY,
  "allow_sharing_character_configs" BOOLEAN NOT NULL,
  "allow_sharing_figure_records" BOOLEAN NOT NULL,
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  "version" INTEGER NOT NULL
);
