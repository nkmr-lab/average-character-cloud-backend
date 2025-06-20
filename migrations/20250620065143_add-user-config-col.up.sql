ALTER TABLE "public"."user_configs" ADD COLUMN "random_level" integer NOT NULL DEFAULT 50;
ALTER TABLE "public"."user_configs" ADD COLUMN "shared_proportion" integer NOT NULL DEFAULT 50;
