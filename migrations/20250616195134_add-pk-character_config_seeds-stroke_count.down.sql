ALTER TABLE "public"."character_config_seeds" DROP CONSTRAINT "character";
ALTER TABLE "public"."character_config_seeds" ADD CONSTRAINT "character_config_seeds_pkey" PRIMARY KEY ("character");
