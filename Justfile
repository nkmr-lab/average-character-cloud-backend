sqlx-prepare:
    cargo sqlx prepare -- --lib

add-migrate name:
    ./add-migration.sh "{{name}}"
