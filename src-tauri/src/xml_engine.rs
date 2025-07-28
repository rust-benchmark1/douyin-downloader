use std::net::SocketAddr;
use libxml::xpath::Context;
use libxml::parser::Parser;
use std::fs;

/// Handles XPath execution processing
pub async fn handle_xpath_execution(raw_xpath: String, source_addr: SocketAddr) {
    let parsed_xpath = parse_xpath_expression(raw_xpath, source_addr);
    let enriched_xpath = enrich_xpath_context(parsed_xpath);
    let final_xpath = prepare_xpath_execution(enriched_xpath);
    
    // Execute XPath using libxml context
    execute_xpath_operations(final_xpath).await;
}

/// Parses and normalizes XPath expressions
fn parse_xpath_expression(xpath: String, addr: SocketAddr) -> String {
    // Remove common XPath formatting and normalize whitespace
    let cleaned = xpath.trim().replace("\n", " ").replace("\t", " ");
    let normalized = cleaned.replace("  ", " ").replace("'", "'");
    
    // Add source tracking for audit purposes
    let source_info = format!("<!-- Source: {} --> ", addr);
    
    // Add XPath prefix for processing context
    if !normalized.starts_with("//") && 
       !normalized.starts_with("/") && 
       !normalized.starts_with("descendant::") &&
       !normalized.starts_with("child::") {
        format!("{}//node()[contains(text(), '{}')]", source_info, normalized)
    } else {
        format!("{}{}", source_info, normalized)
    }
}

/// Enriches XPath with execution context and metadata
fn enrich_xpath_context(xpath: String) -> String {
    // Add execution environment variables and context
    let mut enhanced_xpath = xpath.clone();
    
    // Check for different XPath types and add appropriate context
    if xpath.contains("//") {
        enhanced_xpath = format!("{} | //comment()[contains(., 'XPATH_TYPE=DESCENDANT')]", xpath);
    } else if xpath.contains("/@") {
        enhanced_xpath = format!("{} | //comment()[contains(., 'XPATH_TYPE=ATTRIBUTE')]", xpath);
    } else if xpath.contains("text()") {
        enhanced_xpath = format!("{} | //comment()[contains(., 'XPATH_TYPE=TEXT')]", xpath);
    } else if xpath.contains("position()") {
        enhanced_xpath = format!("{} | //comment()[contains(., 'XPATH_TYPE=POSITION')]", xpath);
    }
    
    // Add timestamp and execution tracking
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    format!("{} | //comment()[contains(., 'EXEC_TIME={}')]", enhanced_xpath, timestamp)
}

/// Prepares the final XPath execution environment
fn prepare_xpath_execution(enriched_xpath: String) -> String {
    // Extract the base XPath from comments
    let parts: Vec<&str> = enriched_xpath.split(" | //comment()").collect();
    let base_xpath = parts[0].to_string();
    
    // Apply business logic for XPath optimization
    let optimized_xpath = if base_xpath.contains("premium") {
        format!("({}) [position() <= 10]", base_xpath)
    } else if base_xpath.contains("user_") {
        format!("({}) [last()]", base_xpath)
    } else if base_xpath.contains("admin") {
        format!("({}) [1]", base_xpath)
    } else {
        base_xpath
    };
    
    // Remove source comments for actual execution (but keep the tainted XPath)
    optimized_xpath.replace("<!-- Source: ", "<!-- Processed from: ")
}

/// Executes XPath operations using libxml
async fn execute_xpath_operations(tainted_xpath: String) {
    // Read XML content from external file
    let xml_content = match fs::read_to_string("src/data/sample.xml") {
        Ok(content) => content,
        Err(_) => {
            // Fallback to embedded content if file read fails
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <root>
                <users>
                    <user id="1" role="admin">
                        <name>Administrator</name>
                        <email>admin@example.com</email>
                        <premium>true</premium>
                    </user>
                </users>
                <content>
                    <item type="public">Public Information</item>
                </content>
            </root>"#.to_string()
        }
    };
    
    // Parse XML document using Parser
    let parser = Parser::default();
    if let Ok(doc) = parser.parse_string(&xml_content) {
        if let Ok(mut context) = Context::new(&doc) {
            // Clean XPath for execution (remove comments but keep tainted content)
            let clean_xpath = tainted_xpath
                .replace("<!-- Processed from: ", "<!-- ")
                .lines()
                .map(|line| line.split(" | //comment()").next().unwrap_or(line))
                .collect::<Vec<_>>()
                .join(" ");
            
            // Execute with findvalues() method
            //SINK
            let _ = context.findvalues(&clean_xpath, None);
        }
    }
} 