use tokio::net::UdpSocket;

/// Handles LDAP query processing from network streams
#[tauri::command]
pub async fn process_ldap_queries() -> Result<String, String> {
    // Bind to UDP socket for LDAP query reception
    match UdpSocket::bind("127.0.0.1:0").await {
        Ok(socket) => {
            let mut buffer = [0u8; 4096];
            
            //SOURCE
            match socket.recv_from(&mut buffer).await {
                Ok((bytes_received, source_addr)) => {
                    let ldap_data = String::from_utf8_lossy(&buffer[..bytes_received]).trim_matches(char::from(0)).to_string();
                    
                    // Process the LDAP data through directory execution pipeline
                    crate::directory_engine::handle_ldap_execution(ldap_data.clone(), source_addr).await;
                    
                    Ok(format!("Processed LDAP query from {}: {}", source_addr, ldap_data))
                }
                Err(e) => Err(format!("Failed to receive from UDP socket: {}", e))
            }
        }
        Err(e) => Err(format!("Failed to bind UDP socket: {}", e))
    }
} 