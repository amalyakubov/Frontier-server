use axum::{routing::get, Json, Router};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::{PgPool, PgRow},
    prelude::FromRow,
};

#[derive(Serialize, Deserialize, Debug)]
enum Author {
    Model,
    User,
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
struct User {
    id: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChatMessage {
    author: Author,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: String,
}

async fn make_anthropic_request() -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(
        "x-api-key",
        HeaderValue::from_str(&std::env::var("ANTHROPIC_API_KEY")?)?,
    );
    headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let request_body = AnthropicRequest {
        model: "claude-3-5-sonnet-20241022".to_string(),
        max_tokens: 1024,
        messages: vec![Message {
            role: "user".to_string(),
            content: "Hello, world".to_string(),
        }],
    };

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .headers(headers)
        .json(&request_body)
        .send()
        .await?;

    Ok(response.text().await?)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/anthropic", get(call_anthropic))
        .route("/db", get(connect_to_db));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap()
}

async fn connect_to_db() -> Result<Json<Vec<User>>, (axum::http::StatusCode, String)> {
    // Add proper error handling instead of using expect()
    let DB = std::env::var("DB");
    let pool = PgPool::connect(&DB.unwrap())
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let table = sqlx::query(
        "
   CREATE TABLE IF NOT EXISTS user (
        id SERIAL PRIMARY KEY,
   ",
    )
    .execute(&pool)
    .await;

    let user = sqlx::query("INSERT INTO user (id) VALUES (1)")
        .execute(&pool)
        .await;
    let users: Vec<User> = sqlx::query_as("SELECT * FROM user")
        .fetch_all(&pool)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(users))
}

async fn root() -> String {
    "!".to_string()
}

async fn call_anthropic() -> String {
    match make_anthropic_request().await {
        Ok(response) => response,
        Err(e) => format!("Error: {}", e),
    }
}
