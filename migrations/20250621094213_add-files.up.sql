CREATE TABLE "public"."files" (
  "id" VARCHAR(64) PRIMARY KEY,
  "user_id" VARCHAR(64) NOT NULL,
  "key" VARCHAR(255) NOT NULL,
  "mime_type" VARCHAR(255) NOT NULL,
  "size" INTEGER NOT NULL,
  "verified" BOOLEAN NOT NULL,
  "created_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  "updated_at" TIMESTAMP WITH TIME ZONE NOT NULL,
  "version" INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX "files_user_id_idx" ON "public"."files" ("user_id");
