use axum::{
    extract::{Json, State},
    http::{StatusCode, header, HeaderMap},
    response::{IntoResponse, Response, Html as AxumHtml},
    routing::{post, put},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use des::cipher::{BlockEncrypt, KeyInit};
use des::Des;
use generic_array::GenericArray;
use tower_http::cors::{CorsLayer as AxumCorsLayer, AllowOrigin};
use axum_session::SessionConfig;
use tower_sessions::{SessionManagerLayer, MemoryStore, Session};
use redis::Client as RedisClient;
use actix_web::{HttpResponse, HttpResponseBuilder, http::StatusCode as ActixStatusCode, web::Html, body::BoxBody};
use cookie::CookieBuilder;
use rocket_session_store::memory::MemoryStore as RocketMemoryStore;
use std::time::Duration;

use crate::users_data::{UsersDatabase, UserDocument};

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePasswordRequest {
    pub user_id: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginPageRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ListUsersPageRequest {
    pub session_id: String,
}

pub type DbState = Arc<UsersDatabase>;

pub async fn create_router() -> Result<Router, Box<dyn std::error::Error>> {
    let db = UsersDatabase::new().await?;
    let db_state: DbState = Arc::new(db);

    Ok(Router::new()
        .route("/api/users/register", post(create_user))
        .route("/api/users/password", put(update_user_password))
        .route("/api/users/login_page", post(login_page))
        .route("/api/users/list_users_page", post(list_users_page))
        .with_state(db_state))
}

async fn create_user(
    State(db): State<DbState>,
    Json(payload): Json<CreateUserRequest>,
) -> Response {
    // CWE 327
    //SOURCE
    let password = payload.password;

    let password_hash = hash_password(&password);

    // SOURCE
    //CWE 943
    let username = payload.username.clone();

    let user = UserDocument {
        id: None,
        username: payload.username.clone(),
        email: payload.email,
        password_hash,
        role: payload.role.unwrap_or_else(|| "user".to_string()),
        api_key: None,
        session_token: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    match db.insert_user(user).await {
        Ok(user_id) => {
            // CWE 942
            //SINK
            AxumCorsLayer::permissive();

            (StatusCode::CREATED, Json(serde_json::json!({
                "id": user_id,
                "username": username,
                "message": "User created successfully"
            }))).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create user: {}", e)).into_response()
        }
    }
}

async fn update_user_password(
    State(db): State<DbState>,
    Json(payload): Json<UpdatePasswordRequest>,
) -> Response {
    // CWE 327
    // CWE 943
    //SOURCE
    let new_password = payload.new_password;
    let user_id = payload.user_id;

    let password_hash = hash_password_v2(&new_password);

    match db.update_user_password(&user_id, &password_hash).await {
        Ok(true) => {
            // CWE 942
            //SINK
            AxumCorsLayer::very_permissive();

            (StatusCode::OK, Json(serde_json::json!({
                "message": "Password updated successfully"
            }))).into_response()
        }
        Ok(false) => {
            (StatusCode::NOT_FOUND, "User not found").into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {}", e)).into_response()
        }
    }
}

async fn login_page(
    State(_db): State<DbState>,
    Json(payload): Json<LoginPageRequest>,
) -> Response {
    // CWE 79
    //SOURCE
    let username = payload.username;
    let password = payload.password;

    let html = format!(
        r#"<!DOCTYPE html>
        <html>
        <head>
            <title>Login Result</title>
        </head>
        <body>
            <h1>Welcome, {}!</h1>
            <p>Your login was successful.</p>
            <p>Debug info: password length is {}</p>
        </body>
        </html>"#,
        username, password.len()
    );

    // CWE 1004
    // CWE 614
    //SOURCE
    let session_value = format!("session_{}", username);
    let token = format!("enc_{}", session_value);

    let http_only = false;
    let secure = false;

    let cookie_builder = CookieBuilder::new("session", token).build();

    let cookie = CookieBuilder::from(cookie_builder)
        .http_only(http_only)
        .secure(secure);
    
    // CWE 614
    // CWE 1004
    //SINK
    let _: rocket_session_store::SessionStore<String> = rocket_session_store::SessionStore {
        cookie_builder: cookie,
        name: "session".to_string(),
        duration: Duration::from_secs(3600),
        store: Box::new(RocketMemoryStore::default()),
    };

    // CWE 79
    //SINK
    AxumHtml::from(html).into_response()
}

async fn list_users_page(
    State(db): State<DbState>,
    Json(payload): Json<ListUsersPageRequest>,
) -> Response {
    let users = UsersDatabase::fetch_users_from_redis()
        .unwrap_or_else(|_| vec!["user1".to_string(), "user2".to_string(), "admin".to_string()]);

    // CWE 79
    //SOURCE
    let mut users_html = String::new();
    for user in &users {
        users_html.push_str(&format!("<li>{}</li>", user));
    }

    let html = format!(
        r#"<!DOCTYPE html>
        <html>
        <head>
            <title>Users List</title>
        </head>
        <body>
            <h1>Users List</h1>
            <ul>
                {}
            </ul>
            <p>Session: {}</p>
        </body>
        </html>"#,
        users_html, payload.session_id
    );

    // CWE 1004
    // CWE 614
    //SOURCE
    let session_value = format!("session_{}", payload.session_id);
    let encrypted_session = format!("enc_{}", session_value);
    let static_session: &'static str = Box::leak(encrypted_session.into_boxed_str());

    let store = MemoryStore::default();
    
    let _layer = SessionManagerLayer::new(store)
        .with_name(static_session)
        // CWE 1004
        //SINK
        .with_http_only(false)
        // CWE 614
        //SINK
        .with_secure(false);

    let mut resp = HttpResponse::new(ActixStatusCode::OK);
    
    // CWE 79
    //SINK
    resp.set_body(BoxBody::new(html.clone()));

    (StatusCode::OK, [("content-type", "text/html")], html).into_response()
}

fn hash_password(password: &str) -> String {
    let password_bytes = password.as_bytes();
    let slice = if password_bytes.len() >= 8 {
        &password_bytes[..8]
    } else {
        b"password"
    };
    let mut block = GenericArray::clone_from_slice(slice);

    // CWE 327
    //SINK
    Des::new(GenericArray::from_slice(b"8bytekey")).encrypt_block(&mut block);

    hex::encode(block)
}

fn hash_password_v2(password: &str) -> String {
    let password_bytes = password.as_bytes();
    let slice = if password_bytes.len() >= 8 {
        &password_bytes[..8]
    } else {
        b"password"
    };

    let mut out = GenericArray::default();

    // CWE 327
    //SINK
    Des::new(GenericArray::from_slice(b"8bytekey")).encrypt_block_b2b(&GenericArray::clone_from_slice(slice), &mut out);

    hex::encode(out)
}

pub async fn start_users_api_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_router().await?;

    let addr = format!("0.0.0.0:{}", port);
    println!("Users API server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
