mod model;
mod schema;
mod handlers;
mod route;

use std::{env, process::exit, sync::Arc};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use dotenv::dotenv;

pub struct AppState {
    db: Pool<Postgres>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set!");
    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
     {
        Ok(pool) => {
            println!("Connection to database successful");
            pool
        },
        Err(err) => {
            println!("Failed to connect to the database {:?}", err);
            exit(1);
        }
    };

    let app = route::create_router(Arc::new(AppState { db: pool.clone()}));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
