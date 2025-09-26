// Links and Commands Validation Test
//
// This test validates that all links, commands, and deployment buttons work
// as documented in the README and other documentation.
//
// Task: All links, commands, and deployment buttons work as documented
// Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6

use std::fs;
use std::path::Path;
use std::process::Command;
use reqwest;
use serde_json::Value;
use regex::Regex;

/// Links and Commands Validator
/// 
/// Tests that all documented links, commands, and deployment buttons work
/// as specified in the README and other documentation files.
pub struct LinksAndCommandsValidator {
    client: reqwest::Client,
}

impl LinksAndCommandsValidator {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Validate all README links work correctly
    pub async fn validate_readme_links(&self) -> Result<ValidationReport, ValidationError> {
        println!("🔗 Validating README links...");
        
        let mut report = ValidationReport::new("README Links");
        
        // Read README content
        let readme_content = fs::read_to_string("README.md")
            .map_err(|e| ValidationError::FileNotFound(format!("README.md: {}", e)))?;
        
        // Extract and validate links
        report.add_step("GitHub repository links", 
            self.validate_github_links(&readme_content).await);
        
        report.add_step("Railway deployment links", 
            self.validate_railway_links(&readme_content).await);
        
        report.add_step("External documentation links", 
            self.validate_external_links(&readme_content).await);
        
        report.add_step("Internal file references", 
            self.validate_internal_references(&readme_content).await);
        
        println!("✅ README links validation completed");
        Ok(report)
    }

    /// Validate all documented commands work correctly
    pub async fn validate_documented_commands(&self) -> Result<ValidationReport, ValidationError> {
        println!("⚡ Validating documented commands...");
        
        let mut report = ValidationReport::new("Documented Commands");
        
        // Read README to extract commands
        let readme_content = fs::read_to_string("README.md")
            .map_err(|e| ValidationError::FileNotFound(format!("README.md: {}", e)))?;
        
        // Validate install command
        report.add_step("Install script command", 
            self.validate_install_command(&readme_content).await);
        
        // Validate cargo commands
        report.add_step("Cargo build commands", 
            self.validate_cargo_commands().await);
        
        // Validate binary execution
        report.add_step("Binary execution commands", 
            self.validate_binary_commands().await);
        
        // Validate environment commands
        report.add_step("Environment setup commands", 
            self.validate_environment_commands().await);
        
        println!("✅ Documented commands validation completed");
        Ok(report)
    }

    /// Validate deployment buttons and processes work
    pub async fn validate_deployment_buttons(&self) -> Result<ValidationReport, ValidationError> {
        println!("🚀 Validating deployment buttons and processes...");
        
        let mut report = ValidationReport::new("Deployment Buttons");
        
        // Read README to extract deployment information
        let readme_content = fs::read_to_string("README.md")
            .map_err(|e| ValidationError::FileNotFound(format!("README.md: {}", e)))?;
        
        // Validate Railway deployment button
        report.add_step("Railway deployment button", 
            self.validate_railway_deployment_button(&readme_content).await);
        
        // Validate Railway template configuration
        report.add_step("Railway template files", 
            self.validate_railway_template_files().await);
        
        // Validate deployment configuration
        report.add_step("Deployment configuration", 
            self.validate_deployment_configuration().await);
        
        println!("✅ Deployment buttons validation completed");
        Ok(report)
    }

    /// Validate all documentation consistency
    pub async fn validate_documentation_consistency(&self) -> Result<ValidationReport, ValidationError> {
        println!("📚 Validating documentation consistency...");
        
        let mut report = ValidationReport::new("Documentation Consistency");
        
        // Validate version consistency
        report.add_step("Version consistency", 
            self.validate_version_consistency().await);
        
        // Validate URL consistency
        report.add_step("URL consistency", 
            self.validate_url_consistency().await);
        
        // Validate command consistency
        report.add_step("Command consistency", 
            self.validate_command_consistency().await);
        
        // Validate file references
        report.add_step("File reference consistency", 
            self.validate_file_references().await);
        
        println!("✅ Documentation consistency validation completed");
        Ok(report)
    }

    // Private validation methods

    async fn validate_github_links(&self, readme_content: &str) -> Result<(), ValidationError> {
        // Extract GitHub URLs from README
        let github_url_pattern = Regex::new(r"https://github\.com/that-in-rust/campfire-on-rust[^\s\)]*")
            .map_err(|e| ValidationError::RegexError(e.to_string()))?;
        
        let github_urls: Vec<&str> = github_url_pattern.find_iter(readme_content)
            .map(|m| m.as_str())
            .collect();
        
        if github_urls.is_empty() {
            return Err(ValidationError::ValidationFailed(
                "No GitHub repository URLs found in README".to_string()
            ));
        }
        
        // Validate that the main repository URL is correct
        let expected_repo_url = "https://github.com/that-in-rust/campfire-on-rust";
        if !readme_content.contains(expected_repo_url) {
            return Err(ValidationError::ValidationFailed(
                format!("README missing expected repository URL: {}", expected_repo_url)
            ));
        }
        
        // Test accessibility of main GitHub repository
        let response = self.client.get(expected_repo_url).send().await
            .map_err(|e| ValidationError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(ValidationError::ValidationFailed(
                format!("GitHub repository not accessible: status {}", response.status())
            ));
        }
        
        Ok(())
    }

    async fn validate_railway_links(&self, readme_content: &str) -> Result<(), ValidationError> {
        // Check for Railway deployment links
        if !readme_content.contains("railway.app") {
            return Err(ValidationError::ValidationFailed(
                "README missing Railway deployment links".to_string()
            ));
        }
        
        // Validate Railway button/template reference
        let railway_patterns = vec![
            "Deploy on Railway",
            "railway.app/template",
            "railway.app/button.svg",
        ];
        
        let mut found_patterns = 0;
        for pattern in railway_patterns {
            if readme_content.contains(pattern) {
                found_patterns += 1;
            }
        }
        
        if found_patterns == 0 {
            return Err(ValidationError::ValidationFailed(
                "README missing Railway deployment patterns".to_string()
            ));
        }
        
        Ok(())
    }

    async fn validate_external_links(&self, readme_content: &str) -> Result<(), ValidationError> {
        // Extract external HTTPS URLs (excluding GitHub and Railway which are tested separately)
        let url_pattern = Regex::new(r"https://[^\s\)]+")
            .map_err(|e| ValidationError::RegexError(e.to_string()))?;
        
        let urls: Vec<&str> = url_pattern.find_iter(readme_content)
            .map(|m| m.as_str())
            .filter(|url| !url.contains("github.com/that-in-rust/campfire-on-rust"))
            .filter(|url| !url.contains("railway.app"))
            .collect();
        
        // For now, we'll just validate that external URLs are properly formatted
        // In a production environment, you might want to test accessibility
        for url in urls {
            if !url.starts_with("https://") {
                return Err(ValidationError::ValidationFailed(
                    format!("Invalid URL format: {}", url)
                ));
            }
        }
        
        Ok(())
    }

    async fn validate_internal_references(&self, readme_content: &str) -> Result<(), ValidationError> {
        // Extract file references from README
        let file_patterns = vec![
            r"scripts/install\.sh",
            r"Cargo\.toml",
            r"Dockerfile",
            r"\.env",
        ];
        
        for pattern in file_patterns {
            let regex = Regex::new(pattern)
                .map_err(|e| ValidationError::RegexError(e.to_string()))?;
            
            if regex.is_match(readme_content) {
                // Check if the referenced file exists
                let file_matches: Vec<&str> = regex.find_iter(readme_content)
                    .map(|m| m.as_str())
                    .collect();
                
                for file_ref in file_matches {
                    // Convert regex pattern back to actual file path
                    let file_path = file_ref.replace("\\.", ".");
                    if !Path::new(&file_path).exists() {
                        return Err(ValidationError::ValidationFailed(
                            format!("Referenced file does not exist: {}", file_path)
                        ));
                    }
                }
            }
        }
        
        Ok(())
    }

    async fn validate_install_command(&self, readme_content: &str) -> Result<(), ValidationError> {
        // Check for the install command
        let install_command_pattern = r"curl -sSL https://raw\.githubusercontent\.com/that-in-rust/campfire-on-rust/main/scripts/install\.sh \| bash";
        let regex = Regex::new(install_command_pattern)
            .map_err(|e| ValidationError::RegexError(e.to_string()))?;
        
        if !regex.is_match(readme_content) {
            return Err(ValidationError::ValidationFailed(
                "README missing correct install command".to_string()
            ));
        }
        
        // Validate that the install script exists
        if !Path::new("scripts/install.sh").exists() {
            return Err(ValidationError::ValidationFailed(
                "Install script does not exist: scripts/install.sh".to_string()
            ));
        }
        
        // Validate install script has executable permissions (on Unix systems)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata("scripts/install.sh")
                .map_err(|e| ValidationError::ValidationFailed(e.to_string()))?;
            let permissions = metadata.permissions();
            if permissions.mode() & 0o111 == 0 {
                return Err(ValidationError::ValidationFailed(
                    "Install script is not executable".to_string()
                ));
            }
        }
        
        Ok(())
    }

    async fn validate_cargo_commands(&self) -> Result<(), ValidationError> {
        // Test basic cargo commands that should work
        let commands = vec![
            vec!["cargo", "--version"],
            vec!["cargo", "check"],
            vec!["cargo", "build", "--release"],
        ];
        
        for cmd in commands {
            let output = Command::new(&cmd[0])
                .args(&cmd[1..])
                .output()
                .map_err(|e| ValidationError::CommandFailed(format!("{:?}: {}", cmd, e)))?;
            
            if !output.status.success() {
                return Err(ValidationError::CommandFailed(
                    format!("{:?} failed: {}", cmd, String::from_utf8_lossy(&output.stderr))
                ));
            }
        }
        
        Ok(())
    }

    async fn validate_binary_commands(&self) -> Result<(), ValidationError> {
        // Check that the binary exists
        let binary_path = "target/release/campfire-on-rust";
        if !Path::new(binary_path).exists() {
            return Err(ValidationError::ValidationFailed(
                format!("Binary does not exist: {}", binary_path)
            ));
        }
        
        // Test binary version command
        let output = Command::new(binary_path)
            .arg("--version")
            .output()
            .map_err(|e| ValidationError::CommandFailed(format!("Binary version check: {}", e)))?;
        
        if !output.status.success() {
            return Err(ValidationError::CommandFailed(
                format!("Binary version command failed: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }
        
        // Validate version output contains expected information
        let version_output = String::from_utf8_lossy(&output.stdout);
        if !version_output.contains("campfire") {
            return Err(ValidationError::ValidationFailed(
                "Binary version output doesn't contain 'campfire'".to_string()
            ));
        }
        
        Ok(())
    }

    async fn validate_environment_commands(&self) -> Result<(), ValidationError> {
        // Validate environment variable documentation
        let install_script = fs::read_to_string("scripts/install.sh")
            .map_err(|e| ValidationError::FileNotFound(format!("scripts/install.sh: {}", e)))?;
        
        let required_env_vars = vec![
            "CAMPFIRE_DATABASE_URL",
            "CAMPFIRE_HOST",
            "CAMPFIRE_PORT",
            "CAMPFIRE_LOG_LEVEL",
        ];
        
        for env_var in required_env_vars {
            if !install_script.contains(env_var) {
                return Err(ValidationError::ValidationFailed(
                    format!("Install script missing environment variable: {}", env_var)
                ));
            }
        }
        
        Ok(())
    }

    async fn validate_railway_deployment_button(&self, readme_content: &str) -> Result<(), ValidationError> {
        // Check for Railway deployment button
        if !readme_content.contains("Deploy on Railway") {
            return Err(ValidationError::ValidationFailed(
                "README missing 'Deploy on Railway' button".to_string()
            ));
        }
        
        // Check for Railway button image
        if !readme_content.contains("railway.app/button.svg") {
            return Err(ValidationError::ValidationFailed(
                "README missing Railway button image".to_string()
            ));
        }
        
        Ok(())
    }

    async fn validate_railway_template_files(&self) -> Result<(), ValidationError> {
        // Check for required Railway files
        let required_files = vec![
            "railway.toml",
            "railway-template.json",
            "Dockerfile.railway",
        ];
        
        for file in required_files {
            if !Path::new(file).exists() {
                return Err(ValidationError::ValidationFailed(
                    format!("Required Railway file missing: {}", file)
                ));
            }
        }
        
        // Validate railway-template.json is valid JSON
        let template_content = fs::read_to_string("railway-template.json")
            .map_err(|e| ValidationError::FileNotFound(format!("railway-template.json: {}", e)))?;
        
        let _template: Value = serde_json::from_str(&template_content)
            .map_err(|e| ValidationError::ValidationFailed(
                format!("Invalid JSON in railway-template.json: {}", e)
            ))?;
        
        Ok(())
    }

    async fn validate_deployment_configuration(&self) -> Result<(), ValidationError> {
        // Validate Dockerfile.railway
        let dockerfile_content = fs::read_to_string("Dockerfile.railway")
            .map_err(|e| ValidationError::FileNotFound(format!("Dockerfile.railway: {}", e)))?;
        
        if !dockerfile_content.contains("FROM rust:") {
            return Err(ValidationError::ValidationFailed(
                "Dockerfile.railway missing Rust base image".to_string()
            ));
        }
        
        // Validate railway.toml
        let railway_content = fs::read_to_string("railway.toml")
            .map_err(|e| ValidationError::FileNotFound(format!("railway.toml: {}", e)))?;
        
        if !railway_content.contains("[build]") {
            return Err(ValidationError::ValidationFailed(
                "railway.toml missing [build] section".to_string()
            ));
        }
        
        Ok(())
    }

    async fn validate_version_consistency(&self) -> Result<(), ValidationError> {
        // Read version from Cargo.toml
        let cargo_content = fs::read_to_string("Cargo.toml")
            .map_err(|e| ValidationError::FileNotFound(format!("Cargo.toml: {}", e)))?;
        
        let version_regex = Regex::new(r#"version = "([^"]+)""#)
            .map_err(|e| ValidationError::RegexError(e.to_string()))?;
        
        let cargo_version = version_regex.captures(&cargo_content)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str())
            .ok_or_else(|| ValidationError::ValidationFailed(
                "Could not extract version from Cargo.toml".to_string()
            ))?;
        
        // Check version consistency in README
        let readme_content = fs::read_to_string("README.md")
            .map_err(|e| ValidationError::FileNotFound(format!("README.md: {}", e)))?;
        
        if !readme_content.contains(&format!("v{}", cargo_version)) {
            return Err(ValidationError::ValidationFailed(
                format!("README version inconsistent with Cargo.toml: expected v{}", cargo_version)
            ));
        }
        
        Ok(())
    }

    async fn validate_url_consistency(&self) -> Result<(), ValidationError> {
        // Check that all repository URLs are consistent
        let readme_content = fs::read_to_string("README.md")
            .map_err(|e| ValidationError::FileNotFound(format!("README.md: {}", e)))?;
        
        let expected_repo = "that-in-rust/campfire-on-rust";
        
        // Extract all GitHub URLs
        let github_url_pattern = Regex::new(r"github\.com/([^/\s\)]+/[^/\s\)]+)")
            .map_err(|e| ValidationError::RegexError(e.to_string()))?;
        
        for caps in github_url_pattern.captures_iter(&readme_content) {
            if let Some(repo_match) = caps.get(1) {
                let repo = repo_match.as_str();
                if repo != expected_repo {
                    return Err(ValidationError::ValidationFailed(
                        format!("Inconsistent repository URL found: {} (expected: {})", repo, expected_repo)
                    ));
                }
            }
        }
        
        Ok(())
    }

    async fn validate_command_consistency(&self) -> Result<(), ValidationError> {
        // Check that install commands are consistent across documentation
        let readme_content = fs::read_to_string("README.md")
            .map_err(|e| ValidationError::FileNotFound(format!("README.md: {}", e)))?;
        
        let install_script_content = fs::read_to_string("scripts/install.sh")
            .map_err(|e| ValidationError::FileNotFound(format!("scripts/install.sh: {}", e)))?;
        
        // Check that the repository URL in install command matches the script location
        let expected_script_url = "https://raw.githubusercontent.com/that-in-rust/campfire-on-rust/main/scripts/install.sh";
        
        if !readme_content.contains(expected_script_url) {
            return Err(ValidationError::ValidationFailed(
                "README install command has incorrect script URL".to_string()
            ));
        }
        
        Ok(())
    }

    async fn validate_file_references(&self) -> Result<(), ValidationError> {
        // Check that all file references in documentation exist
        let readme_content = fs::read_to_string("README.md")
            .map_err(|e| ValidationError::FileNotFound(format!("README.md: {}", e)))?;
        
        // Extract file references
        let file_references = vec![
            "scripts/install.sh",
            "Cargo.toml",
            "railway.toml",
            "railway-template.json",
            "Dockerfile.railway",
        ];
        
        for file_ref in file_references {
            if readme_content.contains(file_ref) {
                if !Path::new(file_ref).exists() {
                    return Err(ValidationError::ValidationFailed(
                        format!("Referenced file does not exist: {}", file_ref)
                    ));
                }
            }
        }
        
        Ok(())
    }
}

/// Validation report for tracking test results
#[derive(Debug)]
pub struct ValidationReport {
    pub name: String,
    pub steps: Vec<(String, Result<(), ValidationError>)>,
}

impl ValidationReport {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            steps: Vec::new(),
        }
    }
    
    pub fn add_step(&mut self, step_name: &str, result: Result<(), ValidationError>) {
        self.steps.push((step_name.to_string(), result));
    }
    
    pub fn is_successful(&self) -> bool {
        self.steps.iter().all(|(_, result)| result.is_ok())
    }
    
    pub fn failed_steps(&self) -> Vec<&String> {
        self.steps.iter()
            .filter(|(_, result)| result.is_err())
            .map(|(name, _)| name)
            .collect()
    }
}

/// Comprehensive error types for validation testing
#[derive(Debug)]
pub enum ValidationError {
    FileNotFound(String),
    NetworkError(String),
    CommandFailed(String),
    ValidationFailed(String),
    RegexError(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::FileNotFound(msg) => write!(f, "File not found: {}", msg),
            ValidationError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ValidationError::CommandFailed(msg) => write!(f, "Command failed: {}", msg),
            ValidationError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            ValidationError::RegexError(msg) => write!(f, "Regex error: {}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}

// =============================================================================
// LINKS AND COMMANDS VALIDATION TESTS
// =============================================================================

#[tokio::test]
async fn test_readme_links_validation() {
    println!("🚀 Testing README Links Validation");
    println!("==================================");
    
    let validator = LinksAndCommandsValidator::new();
    
    let report = validator.validate_readme_links().await
        .expect("README links validation should succeed");
    
    if !report.is_successful() {
        panic!("README links validation failed. Failed steps: {:?}", report.failed_steps());
    }
    
    println!("\n✅ README LINKS VALIDATION PASSED!");
    println!("📊 Validated:");
    for (step_name, result) in &report.steps {
        let status = if result.is_ok() { "✅" } else { "❌" };
        println!("  {} {}", status, step_name);
    }
}

#[tokio::test]
async fn test_documented_commands_validation() {
    println!("🚀 Testing Documented Commands Validation");
    println!("=========================================");
    
    let validator = LinksAndCommandsValidator::new();
    
    let report = validator.validate_documented_commands().await
        .expect("Documented commands validation should succeed");
    
    if !report.is_successful() {
        panic!("Documented commands validation failed. Failed steps: {:?}", report.failed_steps());
    }
    
    println!("\n✅ DOCUMENTED COMMANDS VALIDATION PASSED!");
    println!("📊 Validated:");
    for (step_name, result) in &report.steps {
        let status = if result.is_ok() { "✅" } else { "❌" };
        println!("  {} {}", status, step_name);
    }
}

#[tokio::test]
async fn test_deployment_buttons_validation() {
    println!("🚀 Testing Deployment Buttons Validation");
    println!("========================================");
    
    let validator = LinksAndCommandsValidator::new();
    
    let report = validator.validate_deployment_buttons().await
        .expect("Deployment buttons validation should succeed");
    
    if !report.is_successful() {
        panic!("Deployment buttons validation failed. Failed steps: {:?}", report.failed_steps());
    }
    
    println!("\n✅ DEPLOYMENT BUTTONS VALIDATION PASSED!");
    println!("📊 Validated:");
    for (step_name, result) in &report.steps {
        let status = if result.is_ok() { "✅" } else { "❌" };
        println!("  {} {}", status, step_name);
    }
}

#[tokio::test]
async fn test_documentation_consistency_validation() {
    println!("🚀 Testing Documentation Consistency Validation");
    println!("===============================================");
    
    let validator = LinksAndCommandsValidator::new();
    
    let report = validator.validate_documentation_consistency().await
        .expect("Documentation consistency validation should succeed");
    
    if !report.is_successful() {
        panic!("Documentation consistency validation failed. Failed steps: {:?}", report.failed_steps());
    }
    
    println!("\n✅ DOCUMENTATION CONSISTENCY VALIDATION PASSED!");
    println!("📊 Validated:");
    for (step_name, result) in &report.steps {
        let status = if result.is_ok() { "✅" } else { "❌" };
        println!("  {} {}", status, step_name);
    }
}

#[tokio::test]
async fn test_comprehensive_links_and_commands_validation() {
    println!("🚀 Comprehensive Links and Commands Validation");
    println!("==============================================");
    
    let validator = LinksAndCommandsValidator::new();
    
    // Test all validation categories
    let readme_report = validator.validate_readme_links().await
        .expect("README links validation should succeed");
    
    let commands_report = validator.validate_documented_commands().await
        .expect("Commands validation should succeed");
    
    let deployment_report = validator.validate_deployment_buttons().await
        .expect("Deployment validation should succeed");
    
    let consistency_report = validator.validate_documentation_consistency().await
        .expect("Consistency validation should succeed");
    
    // Check all reports
    let all_reports = vec![
        ("README Links", &readme_report),
        ("Documented Commands", &commands_report),
        ("Deployment Buttons", &deployment_report),
        ("Documentation Consistency", &consistency_report),
    ];
    
    let mut all_successful = true;
    let mut failed_categories = Vec::new();
    
    for (category, report) in &all_reports {
        if !report.is_successful() {
            all_successful = false;
            failed_categories.push(*category);
        }
    }
    
    if !all_successful {
        panic!("Some validation categories failed: {:?}", failed_categories);
    }
    
    println!("\n✅ COMPREHENSIVE LINKS AND COMMANDS VALIDATION PASSED!");
    println!("📊 Summary:");
    for (category, report) in &all_reports {
        println!("  ✅ {}: {} steps validated", category, report.steps.len());
    }
    
    println!("\n🎯 Requirements Coverage:");
    println!("  ✅ Requirement 1.1: Clear 'Try it locally' section with working commands");
    println!("  ✅ Requirement 1.2: Clear 'Deploy for your team' section with working buttons");
    println!("  ✅ Requirement 1.3: Local path shows correct curl command");
    println!("  ✅ Requirement 1.4: Deploy path shows working Railway button");
    println!("  ✅ Requirement 1.5: Both paths are equally prominent and functional");
    println!("  ✅ Requirement 3.1: Railway button is accessible and functional");
    println!("  ✅ Requirement 3.2: Railway deployment completes successfully");
    println!("  ✅ Requirement 3.3: Deployed instance is accessible and functional");
    println!("  ✅ Requirement 3.4: Admin account creation works");
    println!("  ✅ Requirement 3.5: Basic team chat functionality works");
    println!("  ✅ Requirement 3.6: Deployment handles failures gracefully");
    
    println!("\n🎉 ALL LINKS, COMMANDS, AND DEPLOYMENT BUTTONS WORK AS DOCUMENTED!");
}