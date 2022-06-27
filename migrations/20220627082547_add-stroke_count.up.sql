ALTER TABLE "public"."records" ADD COLUMN "stroke_count" integer NOT NULL DEFAULT 0;
UPDATE "public"."records" SET "stroke_count" = jsonb_array_length("figure"->'strokes');
ALTER TABLE "public"."records" ALTER COLUMN "stroke_count" DROP DEFAULT;
CREATE INDEX "records_stroke_count_idx" ON "records" ("stroke_count");
