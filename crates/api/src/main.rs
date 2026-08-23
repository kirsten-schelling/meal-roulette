use axum::http::Method;
use sqlx::PgPool;
use tower_http::cors::CorsLayer;
mod app;
mod error;
mod features;
mod state;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok(); // load .env into process env; .ok() ignores "no .env file"
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await.unwrap();
    let cors = CorsLayer::new()
        .allow_origin(
            "http://localhost:8080"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
        )
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([axum::http::header::CONTENT_TYPE]);

    let app = app::build(state::AppState { db: pool }).layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("api listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
