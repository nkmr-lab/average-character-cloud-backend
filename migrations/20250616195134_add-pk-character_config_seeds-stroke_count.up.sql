ALTER TABLE "public"."character_config_seeds" DROP CONSTRAINT "character_config_seeds_pkey";
ALTER TABLE "public"."character_config_seeds" ADD PRIMARY KEY ("character", "stroke_count");
