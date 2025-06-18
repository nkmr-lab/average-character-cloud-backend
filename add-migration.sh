#!/usr/bin/env bash
set -euo pipefail

name=$1
now=`date -u "+%Y%m%d%H%M%S"`
current_schema=`mktemp --suffix=.sql`
up_file=`mktemp --suffix=.sql`
down_file=`mktemp --suffix=.sql`
sqlx database reset -y
PGSSLMODE=disable PGPASSWORD=$DATABASE_PASSWORD psqldef -U $DATABASE_USER -h $DATABASE_HOST -p $DATABASE_PORT --export $DATABASE_NAME > $current_schema
psqldef $current_schema < ./schema.sql | grep -v _sqlx_migrations | tail -n +2 > $up_file
psqldef ./schema.sql < $current_schema | sed '/_sqlx_migrations/,/);/d' | tail -n +2 > $down_file
mv $up_file migrations/${now}_${name}.up.sql
mv $down_file migrations/${now}_${name}.down.sql
rm $current_schema
sqlx migrate run
