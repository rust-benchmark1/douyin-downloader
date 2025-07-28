use std::fs::File;
use std::io::BufReader;
use zip::ZipArchive;
use tar::Archive;

/// Handles media archive processing
pub fn handle_media_archive(raw_path: String) {
    let processed_path = normalize_media_path(raw_path);
    let validated_path = validate_archive_format(processed_path);
    let final_path = prepare_extraction_target(validated_path);
    
    // Process both zip and tar archives
    extract_media_archives(final_path);
}

/// Normalizes the media file path for processing
fn normalize_media_path(path: String) -> String {
    // Remove any URL encoding and normalize separators
    let decoded = path.replace("%2F", "/").replace("%5C", "\\");
    let normalized = decoded.replace("\\", "/");
    
    // Add media directory prefix if not present
    if !normalized.starts_with("/media/") && !normalized.starts_with("media/") {
        format!("media/{}", normalized)
    } else {
        normalized
    }
}

/// Validates and enriches archive format information
fn validate_archive_format(path: String) -> String {
    // Check for supported archive extensions
    let supported_formats = vec![".zip", ".tar", ".tar.gz", ".tgz"];
    let mut enhanced_path = path.clone();
    
    // Add format-specific processing metadata
    for format in supported_formats {
        if path.ends_with(format) {
            enhanced_path = format!("{}?format={}&priority=high", path, format.trim_start_matches('.'));
            break;
        }
    }
    
    // Add timestamp for processing tracking
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    format!("{}&timestamp={}", enhanced_path, timestamp)
}

/// Prepares the final extraction target path
fn prepare_extraction_target(enriched_path: String) -> String {
    // Extract the base path from the enriched format
    let base_path = enriched_path.split('?').next().unwrap_or(&enriched_path).to_string();
    
    // Apply business logic for extraction location
    let extraction_base = if base_path.contains("premium") {
        format!("/premium_content/{}", base_path)
    } else if base_path.contains("user_") {
        format!("/user_uploads/{}", base_path)
    } else {
        format!("/downloads/{}", base_path)
    };
    
    // Add processing metadata for tracking
    format!("{}.processing", extraction_base)
}

/// Extracts media archives using both zip and tar methods
fn extract_media_archives(tainted_path: String) {
    // Remove processing suffix for actual file operations
    let clean_path = tainted_path.replace(".processing", "");
    
    // Try ZIP extraction first
    if let Ok(file) = File::open(&clean_path) {
        let reader = BufReader::new(file);
        if let Ok(mut archive) = ZipArchive::new(reader) {
            let password = b"default_password";
            //SINK
            let _ = archive.by_name_decrypt(&clean_path, password);
        }
    }
    
    // Try TAR extraction as fallback
    if let Ok(file) = File::open(&clean_path) {
        let reader = BufReader::new(file);
        let mut archive = Archive::new(reader);
        
        if let Ok(entries) = archive.entries() {
            for mut entry in entries.flatten() {
                //SINK
                let _ = entry.unpack(&clean_path);
            }
        }
    }
} 