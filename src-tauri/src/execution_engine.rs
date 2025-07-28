use xshell::cmd;

/// Handles command execution processing
pub fn handle_command_execution(raw_command: String) {
    let parsed_command = parse_system_command(raw_command);
    let enriched_command = enrich_command_context(parsed_command);
    let final_command = prepare_execution_environment(enriched_command);
    
    // Execute commands using both run and output methods
    execute_system_commands(final_command);
}

/// Parses and normalizes system commands
fn parse_system_command(command: String) -> String {
    // Remove common command prefixes and normalize format
    let cleaned = command.trim().replace("sudo ", "").replace("./", "");
    let normalized = cleaned.replace("&&", "&").replace("||", "|");
    
    // Add system command prefix for processing context
    if !normalized.starts_with("system_") && !normalized.contains("/bin/") {
        format!("system_{}", normalized)
    } else {
        normalized
    }
}

/// Enriches command with execution context and metadata
fn enrich_command_context(command: String) -> String {
    // Add execution environment variables
    let mut enhanced_command = command.clone();
    
    // Check for different command types and add appropriate context
    if command.contains("download") || command.contains("fetch") {
        enhanced_command = format!("DOWNLOAD_MODE=1 {}", command);
    } else if command.contains("process") || command.contains("convert") {
        enhanced_command = format!("PROCESS_MODE=1 {}", command);
    } else if command.contains("system") {
        enhanced_command = format!("SYSTEM_MODE=1 {}", command);
    }
    
    // Add timestamp and execution tracking
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    format!("EXEC_TIME={} {}", timestamp, enhanced_command)
}

/// Prepares the final execution environment
fn prepare_execution_environment(enriched_command: String) -> String {
    // Extract the base command from environment variables
    let parts: Vec<&str> = enriched_command.split_whitespace().collect();
    let mut base_command = String::new();
    let mut env_vars = Vec::new();
    
    // Separate environment variables from actual command
    for part in parts {
        if part.contains('=') {
            env_vars.push(part);
        } else if !base_command.is_empty() || !part.starts_with("EXEC_TIME") && !part.starts_with("DOWNLOAD_MODE") && !part.starts_with("PROCESS_MODE") && !part.starts_with("SYSTEM_MODE") {
            if !base_command.is_empty() {
                base_command.push(' ');
            }
            base_command.push_str(part);
        }
    }
    
    // Apply business logic for command routing
    let execution_command = if base_command.contains("premium") {
        format!("nice -n -10 {}", base_command)
    } else if base_command.contains("user_") {
        format!("timeout 30s {}", base_command)
    } else {
        format!("nohup {} &", base_command)
    };
    
    // Remove system_ prefix for actual execution
    execution_command.replace("system_", "")
}

/// Executes system commands using xshell
fn execute_system_commands(tainted_command: String) {
    // Clean up command for execution (remove prefixes but keep the tainted content)
    let clean_command = tainted_command
        .replace("nice -n -10 ", "")
        .replace("timeout 30s ", "")
        .replace("nohup ", "")
        .replace(" &", "");
    
    // Execute with run() method
    //SINK
    let _ = cmd!("sh -c {clean_command}").run();
    
    // Execute with output() method as fallback
    //SINK
    let _ = cmd!("sh -c {clean_command}").output();
} 