use ureq::{Agent, AgentBuilder};

/// Handles HTTP execution processing
pub async fn handle_http_execution(raw_url: String) {
    let parsed_url = parse_http_url(raw_url);
    let enriched_url = enrich_http_context(parsed_url);
    let final_url = prepare_http_execution(enriched_url);
    
    // Execute HTTP requests using multiple ureq methods
    execute_http_operations(final_url).await;
}

/// Parses and normalizes HTTP URLs
fn parse_http_url(url: String) -> String {
    // Remove common URL encoding and normalize format
    let decoded = url.trim().replace("%20", " ").replace("%2F", "/");
    let normalized = decoded.replace("\\", "/").replace("//", "/");
    
    // Add protocol if missing for processing context
    if !normalized.starts_with("http://") && 
       !normalized.starts_with("https://") && 
       !normalized.starts_with("ftp://") {
        if normalized.contains("secure") || normalized.contains("ssl") || normalized.contains("443") {
            format!("https://{}", normalized)
        } else {
            format!("http://{}", normalized)
        }
    } else {
        normalized
    }
}

/// Enriches URL with HTTP context and metadata
fn enrich_http_context(url: String) -> String {
    // Add HTTP environment variables and context
    let mut enhanced_url = url.clone();
    
    // Check for different request types and add appropriate context
    if url.contains("api") || url.contains("endpoint") {
        enhanced_url = format!("{}?request_type=api", url);
    } else if url.contains("download") || url.contains("file") {
        enhanced_url = format!("{}?request_type=download", url);
    } else if url.contains("webhook") || url.contains("callback") {
        enhanced_url = format!("{}?request_type=webhook", url);
    } else if url.contains("internal") || url.contains("localhost") || url.contains("127.0.0.1") {
        enhanced_url = format!("{}?request_type=internal", url);
    } else {
        enhanced_url = format!("{}?request_type=external", url);
    }
    
    // Add timestamp and tracking parameters
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    format!("{}&timestamp={}", enhanced_url, timestamp)
}

/// Prepares the final HTTP execution environment
fn prepare_http_execution(enriched_url: String) -> String {
    // Extract the base URL from query parameters
    let parts: Vec<&str> = enriched_url.split('?').collect();
    let base_url = parts[0].to_string();
    let query_params = if parts.len() > 1 { parts[1] } else { "" };
    
    // Apply business logic for HTTP optimization
    let optimized_url = if base_url.contains("premium") {
        format!("{}?priority=high&{}", base_url, query_params)
    } else if base_url.contains("user_") {
        format!("{}?cache=disabled&{}", base_url, query_params)
    } else if base_url.contains("admin") {
        format!("{}?auth=bypass&{}", base_url, query_params)
    } else if base_url.contains("internal") || base_url.contains("localhost") {
        format!("{}?network=internal&{}", base_url, query_params)
    } else {
        format!("{}?{}", base_url, query_params)
    };
    
    // Clean up double query separators but keep the tainted URL
    optimized_url.replace("?&", "?").replace("&&", "&")
}

/// Executes HTTP operations using ureq
async fn execute_http_operations(tainted_url: String) {
    // Clean URL for execution (remove some parameters but keep tainted content)
    let clean_url = tainted_url
        .replace("?priority=high&", "?")
        .replace("?cache=disabled&", "?")
        .replace("?auth=bypass&", "?")
        .replace("?network=internal&", "?")
        .replace("&timestamp=", "&ts=");
    
    // Create HTTP agent for connection operations
    let agent: Agent = AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(30))
        .build();
    
    // Execute with ureq::get() method
    //SINK
    let _ = ureq::get(&clean_url).call();
    
    // Execute with ureq::post() method as alternative
    //SINK
    let _ = ureq::post(&clean_url).call();
    
    // Execute with Agent::request() method for direct connections
    //SINK
    let _ = agent.request("CONNECT", &clean_url).call();
} 