sqlx-prepare:
    cargo sqlx prepare -- --lib

add-migrate name:
    ./add-migration.sh "{{name}}"

migrate: docker-up
    cargo sqlx migrate run

minio-configure: docker-up
    cargo run -- migrate-storage

docker-up:
    docker compose up -d

serve: migrate minio-configure
    cargo watch -x run
