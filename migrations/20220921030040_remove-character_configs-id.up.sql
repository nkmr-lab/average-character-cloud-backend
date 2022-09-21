ALTER TABLE "public"."character_configs" ALTER COLUMN "user_id" DROP NOT NULL;
ALTER TABLE "public"."character_configs" ALTER COLUMN "character" DROP NOT NULL;
ALTER TABLE "public"."character_configs" DROP CONSTRAINT "character_configs_pkey";
ALTER TABLE "public"."character_configs" ADD primary key ("user_id", "character");
DROP INDEX "public"."character_configs_user_id_idx";
DROP INDEX "public"."character_configs_character_idx";
DROP INDEX "public"."character_configs_user_id_character_idx";
ALTER TABLE "public"."character_configs" DROP COLUMN "id";
