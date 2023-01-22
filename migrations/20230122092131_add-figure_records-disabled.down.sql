DROP INDEX "public"."figure_records_version_idx";
DROP INDEX "public"."figure_records_disabled_idx";
ALTER TABLE "public"."figure_records" DROP COLUMN "version";
ALTER TABLE "public"."figure_records" DROP COLUMN "disabled";
