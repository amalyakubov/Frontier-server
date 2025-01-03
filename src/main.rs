use axum::{
    Router, 
    routing::get,
    Json
};
use serde::{Serialize, Deserialize};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};

#[derive(Serialize, Deserialize, Debug)]
struct MyData {
    name: String,
    age: u32,
    active: bool
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
    headers.insert("x-api-key", HeaderValue::from_str(&std::env::var("ANTHROPIC_API_KEY")?)?);
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
        .route("/data", get(return_json_object))
        .route("/anthropic", get(call_anthropic));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap()
}

async fn root() -> String {
    "!".to_string()
}

async fn return_json_object() -> Json<MyData> {
    let data = MyData {
        name: "Jane Doe".to_string(),
        age: 25, 
        active: false,
    };
    Json(data)
}

async fn call_anthropic() -> String {
    match make_anthropic_request().await {
        Ok(response) => response,
        Err(e) => format!("Error: {}", e),
    }
}