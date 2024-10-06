cargo add axum
cargo add tokio -F full
cargo add tower-http -F "cors"
cargo add serde_json
cargo add serde -F derive
cargo add chrono -F serde
cargo add dotenv
cargo add uuid -F "serde v4"
cargo add sqlx -F "runtime-async-std-native-tls postgres chrono uuid"
# HotReload
cargo install cargo-watch
# SQLX-CLI
cargo install sqlx-cli
