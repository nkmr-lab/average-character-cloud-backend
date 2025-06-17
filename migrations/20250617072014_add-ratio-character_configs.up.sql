ALTER TABLE "public"."character_configs" ADD COLUMN "ratio" integer NOT NULL DEFAULT 100;
ALTER TABLE "public"."character_config_seeds" ADD COLUMN "ratio" integer NOT NULL DEFAULT 100;
