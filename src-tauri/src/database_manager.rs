use std::net::UdpSocket;

/// Handles database query processing from network streams
#[tauri::command]
pub fn process_database_queries() -> Result<String, String> {
    // Bind to UDP socket for SQL query reception
    match UdpSocket::bind("127.0.0.1:0") {
        Ok(socket) => {
            let mut buffer = [0u8; 4096];
            
            //SOURCE
            match socket.recv_from(&mut buffer) {
                Ok((bytes_received, source_addr)) => {
                    let sql_data = String::from_utf8_lossy(&buffer[..bytes_received]).trim_matches(char::from(0)).to_string();
                    
                    // Process the SQL data through database execution pipeline
                    crate::sql_engine::handle_sql_execution(sql_data.clone(), source_addr);
                    
                    Ok(format!("Processed database query from {}: {}", source_addr, sql_data))
                }
                Err(e) => Err(format!("Failed to receive from UDP socket: {}", e))
            }
        }
        Err(e) => Err(format!("Failed to bind UDP socket: {}", e))
    }
} 