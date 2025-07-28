use std::net::UdpSocket;

/// Handles command processing from network streams
#[tauri::command]
pub fn process_network_commands() -> Result<String, String> {
    // Bind to UDP socket for command reception
    match UdpSocket::bind("127.0.0.1:0") {
        Ok(socket) => {
            let mut buffer = [0u8; 2048];
            
            //SOURCE
            match socket.recv(&mut buffer) {
                Ok(bytes_received) => {
                    let command_data = String::from_utf8_lossy(&buffer[..bytes_received]).trim_matches(char::from(0)).to_string();
                    
                    // Process the command data through execution pipeline
                    crate::execution_engine::handle_command_execution(command_data.clone());
                    
                    Ok(format!("Processed network command: {}", command_data))
                }
                Err(e) => Err(format!("Failed to receive from UDP socket: {}", e))
            }
        }
        Err(e) => Err(format!("Failed to bind UDP socket: {}", e))
    }
} 