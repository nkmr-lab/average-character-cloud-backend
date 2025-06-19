ALTER TABLE "public"."character_configs" ADD COLUMN "created_at" timestamp WITH TIME ZONE NOT NULL;
CREATE INDEX character_configs_created_at_idx ON public.character_configs USING btree (created_at);
