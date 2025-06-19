ALTER TABLE "public"."character_configs" ADD COLUMN "disabled" boolean NOT NULL DEFAULT false;
CREATE INDEX "character_configs_user_id_idx" ON "public"."character_configs" ("user_id");
CREATE INDEX "character_configs_character_idx" ON "public"."character_configs" ("character");
CREATE INDEX "character_configs_disabled_idx" ON "public"."character_configs" ("disabled");
