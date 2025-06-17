sqlx-prepare:
    cargo sqlx prepare -- --lib

add-migrate name:
    ./add-migration.sh "{{name}}"

migrate: docker-up
    cargo sqlx migrate run

docker-up:
    docker compose up -d

serve: migrate
    cargo watch -x run
