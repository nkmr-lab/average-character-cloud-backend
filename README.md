# average-character-cloud-backend

## 開発
nixとdirenvが入ってれば大体開発できるはず.

```
$ direnv allow # 基本は初回だけ
$ docker-compose up -d # DB起動
$ sqlx database reset -y # DBマイグレーション(DBのデータぶっ飛びます)
$ cargo run
# open: http://localhost:8080/graphiql
```

## マイグレーション追加

```
# edit schema.sql
$ just add-migrate マイグレーション名
```

## CI落ちた時

```
# 以下の色々をしてみて
$ just sqlx-prepare
$ cargo fmt
$ cargo clippy # エラーが出たところを直す
```
