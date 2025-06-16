#!/usr/bin/env bash
set -eu

name=$1
now=`date -u "+%Y%m%d%H%M%S"`
current_schema=`mktemp --suffix=.sql`
sqlx database reset -y
PGSSLMODE=disable PGPASSWORD=$DATABASE_PASSWORD psqldef -U $DATABASE_USER -h $DATABASE_HOST -p $DATABASE_PORT --export $DATABASE_NAME > $current_schema
psqldef $current_schema < ./schema.sql | grep -v _sqlx_migrations | tail -n +2 > migrations/${now}_${name}.up.sql
psqldef ./schema.sql < $current_schema | sed '/_sqlx_migrations/,/);/d' | tail -n +2 > migrations/${now}_${name}.down.sql
rm $current_schema
sqlx migrate run
