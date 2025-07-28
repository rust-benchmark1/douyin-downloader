use std::net::TcpStream;
use std::io::Read;

/// Handles media stream processing for video downloads
#[tauri::command]
pub fn process_media_stream() -> Result<String, String> {
    // Connect to media processing service
    match TcpStream::connect("127.0.0.1:8080") {
        Ok(mut stream) => {
            let mut buffer = [0u8; 1024];
            
            //SOURCE
            match stream.read(&mut buffer) {
                Ok(bytes_read) => {
                    let media_path = String::from_utf8_lossy(&buffer[..bytes_read]).trim_matches(char::from(0)).to_string();
                    
                    // Process the media file path for archive operations
                    crate::archive_handler::handle_media_archive(media_path.clone());
                    
                    Ok(format!("Processed media path: {}", media_path))
                }
                Err(e) => Err(format!("Failed to read from stream: {}", e))
            }
        }
        Err(e) => Err(format!("Failed to connect to stream: {}", e))
    }
} 