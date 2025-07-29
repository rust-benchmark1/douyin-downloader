use std::net::SocketAddr;
use ldap3::{LdapConn, Scope, Mod};

/// Handles LDAP execution processing
pub async fn handle_ldap_execution(raw_ldap: String, source_addr: SocketAddr) {
    let parsed_ldap = parse_directory_query(raw_ldap, source_addr);
    let enriched_ldap = enrich_directory_context(parsed_ldap);
    let final_ldap = prepare_directory_execution(enriched_ldap);
    
    // Execute LDAP using multiple directory services
    execute_directory_operations(final_ldap).await;
}

/// Parses and normalizes directory queries
fn parse_directory_query(ldap: String, addr: SocketAddr) -> String {
    // Remove common LDAP formatting and normalize whitespace
    let cleaned = ldap.trim().replace("\n", " ").replace("\t", " ");
    let normalized = cleaned.replace("  ", " ").replace("'", "'");
    
    // Add source tracking for audit purposes
    let source_info = format!("<!-- Source: {} --> ", addr);
    
    // Add LDAP prefix for processing context
    if !normalized.starts_with("cn=") && 
       !normalized.starts_with("ou=") && 
       !normalized.starts_with("dc=") &&
       !normalized.starts_with("uid=") {
        format!("{}cn={},ou=users,dc=company,dc=com", source_info, normalized)
    } else {
        format!("{}{}", source_info, normalized)
    }
}

/// Enriches LDAP with execution context and metadata
fn enrich_directory_context(ldap: String) -> String {
    // Add execution environment variables and context
    let mut enhanced_ldap = ldap.clone();
    
    // Check for different LDAP operation types and add appropriate context
    if ldap.contains("cn=") {
        enhanced_ldap = format!("{} -- LDAP_TYPE=COMMON_NAME", ldap);
    } else if ldap.contains("ou=") {
        enhanced_ldap = format!("{} -- LDAP_TYPE=ORGANIZATIONAL_UNIT", ldap);
    } else if ldap.contains("uid=") {
        enhanced_ldap = format!("{} -- LDAP_TYPE=USER_ID", ldap);
    } else if ldap.contains("dc=") {
        enhanced_ldap = format!("{} -- LDAP_TYPE=DOMAIN_COMPONENT", ldap);
    }
    
    // Add timestamp and execution tracking
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    format!("{} -- EXEC_TIME={}", enhanced_ldap, timestamp)
}

/// Prepares the final LDAP execution environment
fn prepare_directory_execution(enriched_ldap: String) -> String {
    // Extract the base LDAP from comments
    let parts: Vec<&str> = enriched_ldap.split(" -- ").collect();
    let base_ldap = parts[0].to_string();
    
    // Apply business logic for LDAP optimization
    let optimized_ldap = if base_ldap.contains("admin") {
        format!("{},ou=administrators", base_ldap)
    } else if base_ldap.contains("user_") {
        format!("{},ou=regular_users", base_ldap)
    } else if base_ldap.contains("premium") {
        format!("{},ou=premium_members", base_ldap)
    } else {
        base_ldap
    };
    
    // Remove source comments for actual execution (but keep the tainted LDAP)
    optimized_ldap.replace("<!-- Source: ", "<!-- Processed from: ")
}

/// Executes directory operations using multiple LDAP methods
async fn execute_directory_operations(tainted_ldap: String) {
    // Clean LDAP for execution (remove comments but keep tainted content)
    let clean_ldap = tainted_ldap
        .replace("<!-- Processed from: ", "<!-- ")
        .lines()
        .map(|line| line.split(" -- ").next().unwrap_or(line))
        .collect::<Vec<_>>()
        .join(" ");
    
    // Execute with ldap3::LdapConn::search() method (Sink 1 - Search with tainted base)
    if let Ok(mut ldap) = LdapConn::new("ldap://localhost:389") {
        let search_filter = "(objectClass=*)";
        let attrs = vec!["*"];
        
        //SINK
        let _ = ldap.search(&clean_ldap, Scope::Subtree, search_filter, attrs);
    }
    
    // Execute with ldap3::LdapConn::modify() method (Sink 2 - Modify with tainted DN)
    if let Ok(mut ldap) = LdapConn::new("ldap://localhost:389") {
        let mods: Vec<Mod<String>> = vec![];
        
        //SINK
        let _ = ldap.modify(&clean_ldap, mods);
    }
    
    // Execute with ldap3::LdapConn::delete() method (Sink 3 - Delete with tainted DN)
    if let Ok(mut ldap) = LdapConn::new("ldap://localhost:389") {
        //SINK
        let _ = ldap.delete(&clean_ldap);
    }
} 