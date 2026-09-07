use api::{app, state::AppState};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok(); // load .env into process env; .ok() ignores "no .env file"
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let state = AppState::connect(&database_url).await.unwrap();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("api listening on http://localhost:3000");
    axum::serve(listener, app::build(state)).await.unwrap();
}
