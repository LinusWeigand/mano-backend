# Commnands

1. Start Projekt with watcher
   cargo watch -q -c -w src/ -x run

2. Log into Postgresql
   psql -h localhost -U linus -p 6500 -d mano

3. Run Migrations
   sqlx migrate run

4. Update Sqlx migration knowledge
   cargo sqlx prepare

5. Add Migration
    sqlx migrate add <name>