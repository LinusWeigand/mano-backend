docker-compose up -d

git reset --hard

git pull

sqlx migrate run

cargo run
