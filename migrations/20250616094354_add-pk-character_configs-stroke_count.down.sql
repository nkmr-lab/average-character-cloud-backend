ALTER TABLE "public"."character_configs" DROP CONSTRAINT "user_id";
ALTER TABLE "public"."character_configs" ADD CONSTRAINT "character_configs_pkey" PRIMARY KEY ("user_id", "character");
