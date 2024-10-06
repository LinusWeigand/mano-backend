mod model;
mod schema;
mod handlers;
mod route;
mod email;
mod utils;

use std::{env, process::exit, sync::Arc};
use email::EmailManager;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use dotenv::dotenv;

pub struct AppState {
    db: Pool<Postgres>,
    email_manager: Arc<EmailManager>,
    url: String
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set!");
    let url = env::var("URL").expect("URL must be set!");

    let smtp_email = env::var("SMTP_EMAIL").expect("SMTP_EMAIL must be set");
    let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");
    let email_manager = match EmailManager::new(&smtp_email, &smtp_password) {
        Ok(manager) => Arc::new(manager),
        Err(e) => {
            eprintln!("Failed to create EmailManager: {:?}", e);
            std::process::exit(1);
        }
    };


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

    let app = route::create_router(Arc::new(
        AppState { db:
            pool.clone(), 
            email_manager: email_manager.clone(),
            url
        }));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
