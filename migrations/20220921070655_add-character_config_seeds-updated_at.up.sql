ALTER TABLE "public"."character_config_seeds" ADD COLUMN "updated_at" timestamp WITH TIME ZONE NOT NULL;
CREATE INDEX "character_config_seeds_updated_at_idx" ON "character_config_seeds" ("updated_at");
