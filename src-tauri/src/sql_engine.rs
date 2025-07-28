use std::net::SocketAddr;
use tiberius::{Client, Config, AuthMethod};
use tokio_util::compat::TokioAsyncWriteCompatExt;

/// Handles SQL execution processing
pub fn handle_sql_execution(raw_sql: String, source_addr: SocketAddr) {
    let parsed_sql = parse_database_query(raw_sql, source_addr);
    let enriched_sql = enrich_query_context(parsed_sql);
    let final_sql = prepare_query_execution(enriched_sql);
    
    // Execute SQL using both query and execute methods
    tokio::spawn(async move {
        execute_database_operations(final_sql).await;
    });
}

/// Parses and normalizes database queries
fn parse_database_query(sql: String, addr: SocketAddr) -> String {
    // Remove common SQL formatting and normalize whitespace
    let cleaned = sql.trim().replace("\n", " ").replace("\t", " ");
    let normalized = cleaned.replace("  ", " ").replace("'", "'");
    
    // Add source tracking for audit purposes
    let source_info = format!("/* Source: {} */ ", addr);
    
    // Add query prefix for processing context
    if !normalized.to_uppercase().starts_with("SELECT") && 
       !normalized.to_uppercase().starts_with("INSERT") && 
       !normalized.to_uppercase().starts_with("UPDATE") && 
       !normalized.to_uppercase().starts_with("DELETE") {
        format!("{}SELECT * FROM users WHERE {}", source_info, normalized)
    } else {
        format!("{}{}", source_info, normalized)
    }
}

/// Enriches query with execution context and metadata
fn enrich_query_context(sql: String) -> String {
    // Add execution environment variables and context
    let mut enhanced_sql = sql.clone();
    
    // Check for different query types and add appropriate context
    if sql.to_uppercase().contains("SELECT") {
        enhanced_sql = format!("{} -- QUERY_TYPE=SELECT", sql);
    } else if sql.to_uppercase().contains("INSERT") {
        enhanced_sql = format!("{} -- QUERY_TYPE=INSERT", sql);
    } else if sql.to_uppercase().contains("UPDATE") {
        enhanced_sql = format!("{} -- QUERY_TYPE=UPDATE", sql);
    } else if sql.to_uppercase().contains("DELETE") {
        enhanced_sql = format!("{} -- QUERY_TYPE=DELETE", sql);
    }
    
    // Add timestamp and execution tracking
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    format!("{} -- EXEC_TIME={}", enhanced_sql, timestamp)
}

/// Prepares the final query execution environment
fn prepare_query_execution(enriched_sql: String) -> String {
    // Extract the base SQL from comments
    let parts: Vec<&str> = enriched_sql.split(" -- ").collect();
    let base_sql = parts[0].to_string();
    
    // Apply business logic for query optimization
    let optimized_sql = if base_sql.to_uppercase().contains("PREMIUM") {
        format!("{} WITH (NOLOCK)", base_sql)
    } else if base_sql.to_uppercase().contains("USER_") {
        format!("{} OPTION (MAXDOP 1)", base_sql)
    } else if base_sql.to_uppercase().contains("ADMIN") {
        format!("{} WITH (READUNCOMMITTED)", base_sql)
    } else {
        base_sql
    };
    
    // Remove source comments for actual execution (but keep the tainted SQL)
    optimized_sql.replace("/* Source: ", "/* Processed from: ")
}

/// Executes database operations using tiberius
async fn execute_database_operations(tainted_sql: String) {
    // Create database configuration
    let mut config = Config::new();
    config.host("localhost");
    config.port(1433);
    config.database("test_db");
    config.authentication(AuthMethod::sql_server("sa", "password"));
    config.trust_cert();
    
    // Attempt to connect to database
    if let Ok(tcp) = tokio::net::TcpStream::connect(config.get_addr()).await {
        let tcp = tcp.compat_write();
        
        if let Ok(mut client) = Client::connect(config, tcp).await {
            // Clean SQL for execution (remove comments but keep tainted content)
            let clean_sql = tainted_sql
                .replace("/* Processed from: ", "/* ")
                .lines()
                .map(|line| line.split(" -- ").next().unwrap_or(line))
                .collect::<Vec<_>>()
                .join(" ");
            
            // Execute with query() method
            //SINK
            let _ = client.query(&clean_sql, &[]).await;
            
            // Execute with execute() method as alternative
            //SINK
            let _ = client.execute(&clean_sql, &[]).await;
        }
    }
} 