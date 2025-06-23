sqlx-prepare:
    cargo sqlx prepare -- --lib

add-migrate name:
    ./add-migration.sh "{{name}}"

migrate: docker-up
    cargo sqlx migrate run

minio-configure: docker-up
    mc alias set minio http://localhost:9000 $AWS_ACCESS_KEY_ID $AWS_SECRET_ACCESS_KEY
    mc mb -p minio/$AVCC_STORAGE_BUCKET

docker-up:
    docker compose up -d

serve: migrate minio-configure
    cargo watch -x run
