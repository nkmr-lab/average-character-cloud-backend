ALTER TABLE "public"."character_configs" DROP CONSTRAINT "character_configs_pkey";
ALTER TABLE "public"."character_configs" ADD PRIMARY KEY ("user_id", "character", "stroke_count");
