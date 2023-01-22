ALTER TABLE "public"."figure_records" ADD COLUMN "version" integer NOT NULL DEFAULT 1;
ALTER TABLE "public"."figure_records" ADD COLUMN "disabled" boolean NOT NULL DEFAULT false;
CREATE INDEX "figure_records_version_idx" ON "figure_records" ("version");
CREATE INDEX "figure_records_disabled_idx" ON "figure_records" ("disabled");
