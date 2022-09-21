ALTER TABLE "public"."character_configs" ADD COLUMN "id" character varying(64) NOT NULL;
ALTER TABLE "public"."character_configs" DROP CONSTRAINT "character_configs_pkey";
ALTER TABLE "public"."character_configs" ADD primary key ("id");
CREATE INDEX character_configs_user_id_idx ON public.character_configs USING btree (user_id);
CREATE INDEX character_configs_character_idx ON public.character_configs USING btree ("character");
CREATE UNIQUE INDEX character_configs_user_id_character_idx ON public.character_configs USING btree (user_id, "character");
