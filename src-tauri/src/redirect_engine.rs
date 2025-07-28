use tide::{Redirect, Response, StatusCode};

/// Handles redirect execution processing
pub async fn handle_redirect_execution(raw_url: String) {
    let parsed_url = parse_redirect_url(raw_url);
    let enriched_url = enrich_redirect_context(parsed_url);
    let final_url = prepare_redirect_execution(enriched_url);
    
    // Execute redirects using both Redirect::new and Response::insert_header methods
    execute_redirect_operations(final_url).await;
}

/// Parses and normalizes redirect URLs
fn parse_redirect_url(url: String) -> String {
    // Remove common URL encoding and normalize format
    let decoded = url.trim().replace("%20", " ").replace("%2F", "/");
    let normalized = decoded.replace("\\", "/").replace("//", "/");
    
    // Add protocol if missing for processing context
    if !normalized.starts_with("http://") && 
       !normalized.starts_with("https://") && 
       !normalized.starts_with("ftp://") {
        if normalized.contains("secure") || normalized.contains("ssl") {
            format!("https://{}", normalized)
        } else {
            format!("http://{}", normalized)
        }
    } else {
        normalized
    }
}

/// Enriches URL with redirect context and metadata
fn enrich_redirect_context(url: String) -> String {
    // Add redirect environment variables and context
    let mut enhanced_url = url.clone();
    
    // Check for different redirect types and add appropriate context
    if url.contains("download") || url.contains("file") {
        enhanced_url = format!("{}?redirect_type=download", url);
    } else if url.contains("login") || url.contains("auth") {
        enhanced_url = format!("{}?redirect_type=auth", url);
    } else if url.contains("external") || url.contains("out") {
        enhanced_url = format!("{}?redirect_type=external", url);
    } else {
        enhanced_url = format!("{}?redirect_type=general", url);
    }
    
    // Add timestamp and tracking parameters
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    format!("{}&timestamp={}", enhanced_url, timestamp)
}

/// Prepares the final redirect execution environment
fn prepare_redirect_execution(enriched_url: String) -> String {
    // Extract the base URL from query parameters
    let parts: Vec<&str> = enriched_url.split('?').collect();
    let base_url = parts[0].to_string();
    let query_params = if parts.len() > 1 { parts[1] } else { "" };
    
    // Apply business logic for redirect optimization
    let optimized_url = if base_url.contains("premium") {
        format!("{}?priority=high&{}", base_url, query_params)
    } else if base_url.contains("user_") {
        format!("{}?cache=enabled&{}", base_url, query_params)
    } else if base_url.contains("admin") {
        format!("{}?secure=true&{}", base_url, query_params)
    } else {
        format!("{}?{}", base_url, query_params)
    };
    
    // Clean up double query separators but keep the tainted URL
    optimized_url.replace("?&", "?").replace("&&", "&")
}

/// Executes redirect operations using tide
async fn execute_redirect_operations(tainted_url: String) {
    // Clean URL for execution (remove some parameters but keep tainted content)
    let clean_url = tainted_url
        .replace("?priority=high&", "?")
        .replace("?cache=enabled&", "?")
        .replace("?secure=true&", "?")
        .replace("&timestamp=", "&ts=");
    
    // Execute with Redirect::new() method
    //SINK
    let _redirect = Redirect::new(&clean_url);
    
    // Execute with Response::insert_header() method as alternative
    let mut response = Response::new(StatusCode::Found);
    //SINK
    response.insert_header("Location", &clean_url);
} 