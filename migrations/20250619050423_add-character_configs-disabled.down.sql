DROP INDEX "public"."character_configs_user_id_idx";
DROP INDEX "public"."character_configs_character_idx";
DROP INDEX "public"."character_configs_disabled_idx";
ALTER TABLE "public"."character_configs" DROP COLUMN "disabled";
