use sqlx::PgPool;
mod app;
mod error;
mod features;
mod state;

#[tokio::main]
async fn main() {
    let pool = PgPool::connect("postgresql://postgres:dev@localhost:5432/meal_roulette")
        .await
        .unwrap();
    let app = app::build(state::AppState { db: pool });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("api listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
