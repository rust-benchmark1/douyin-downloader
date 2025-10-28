#[path = "../users_service.rs"]
mod users_service;

#[path = "../users_data.rs"]
mod users_data;

#[tokio::main]
async fn main() {
    println!("Starting Users API Server on http://localhost:3001");
    println!("Press Ctrl+C to stop\n");
    println!("Available endpoints:");
    println!("  POST http://localhost:3001/api/users/register");
    println!("  PUT  http://localhost:3001/api/users/password");
    println!("  POST http://localhost:3001/api/users/login_page");
    println!("  POST http://localhost:3001/api/users/list_users_page\n");

    if let Err(e) = users_service::start_users_api_server(3001).await {
        eprintln!("Error: {}", e);
    }
}
