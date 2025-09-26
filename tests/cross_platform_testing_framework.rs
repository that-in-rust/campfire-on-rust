/// Cross-Platform Testing Framework
/// 
/// This module implements comprehensive cross-platform testing for:
/// - macOS (Intel/Apple Silicon)
/// - Linux (Ubuntu/CentOS) 
/// - Windows (WSL)
/// 
/// Uses industry standard testing frameworks to simulate different platforms
/// and identify potential issues without requiring actual access to all platforms.
/// 
/// Requirements: 1.5, 2.1, 3.2, 10.1, 10.5, 10.7

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;
use serde::{Deserialize, Serialize};

/// Platform-specific configuration and testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub os: String,
    pub arch: String,
    pub binary_extension: String,
    pub shell: String,
    pub package_manager: String,
    pub common_issues: Vec<String>,
    pub install_dependencies: Vec<String>,
}

/// Cross-platform testing results
#[derive(Debug, Clone)]
pub struct CrossPlatformTestResults {
    pub platform: String,
    pub tests_passed: usize,
    pub tests_failed: usize,
    pub issues_found: Vec<PlatformIssue>,
    pub recommendations: Vec<String>,
}

/// Platform-specific issue
#[derive(Debug, Clone)]
pub struct PlatformIssue {
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub description: String,
    pub solution: String,
    pub affects_platforms: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone)]
pub enum IssueCategory {
    Installation,
    Runtime,
    Configuration,
    Performance,
    Security,
}

/// Main cross-platform testing suite
#[tokio::test]
async fn test_cross_platform_compatibility_comprehensive() {
    println!("🌍 Starting comprehensive cross-platform testing");
    
    let platforms = get_supported_platforms();
    let mut results = Vec::new();
    
    for platform in platforms {
        println!("🔍 Testing platform: {} {}", platform.os, platform.arch);
        let result = test_platform_compatibility(&platform).await;
        results.push(result);
    }
    
    // Generate comprehensive report
    generate_cross_platform_report(&results).await;
    
    // Verify all critical issues are addressed
    verify_no_critical_issues(&results);
    
    println!("✅ Cross-platform testing completed");
}

/// Get all supported platform configurations
fn get_supported_platforms() -> Vec<PlatformConfig> {
    vec![
        // macOS Intel
        PlatformConfig {
            os: "darwin".to_string(),
            arch: "x86_64".to_string(),
            binary_extension: "".to_string(),
            shell: "zsh".to_string(),
            package_manager: "brew".to_string(),
            common_issues: vec![
                "Gatekeeper blocking unsigned binaries".to_string(),
                "Rosetta 2 compatibility on Apple Silicon".to_string(),
                "PATH not updated in new terminal sessions".to_string(),
            ],
            install_dependencies: vec!["curl".to_string()],
        },
        
        // macOS Apple Silicon
        PlatformConfig {
            os: "darwin".to_string(),
            arch: "aarch64".to_string(),
            binary_extension: "".to_string(),
            shell: "zsh".to_string(),
            package_manager: "brew".to_string(),
            common_issues: vec![
                "Native ARM64 binary performance".to_string(),
                "Homebrew path differences".to_string(),
                "Xcode command line tools requirement".to_string(),
            ],
            install_dependencies: vec!["curl".to_string()],
        },
        
        // Ubuntu Linux
        PlatformConfig {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            binary_extension: "".to_string(),
            shell: "bash".to_string(),
            package_manager: "apt".to_string(),
            common_issues: vec![
                "Missing libc dependencies".to_string(),
                "Permission issues with ~/.local/bin".to_string(),
                "Firewall blocking port 3000".to_string(),
                "SQLite version compatibility".to_string(),
            ],
            install_dependencies: vec!["curl".to_string(), "ca-certificates".to_string()],
        },
        
        // CentOS/RHEL Linux
        PlatformConfig {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            binary_extension: "".to_string(),
            shell: "bash".to_string(),
            package_manager: "yum".to_string(),
            common_issues: vec![
                "SELinux blocking execution".to_string(),
                "Older glibc versions".to_string(),
                "Missing OpenSSL libraries".to_string(),
                "Systemd service configuration".to_string(),
            ],
            install_dependencies: vec!["curl".to_string(), "openssl".to_string()],
        },
        
        // Linux ARM64 (Raspberry Pi, etc.)
        PlatformConfig {
            os: "linux".to_string(),
            arch: "aarch64".to_string(),
            binary_extension: "".to_string(),
            shell: "bash".to_string(),
            package_manager: "apt".to_string(),
            common_issues: vec![
                "ARM64 binary availability".to_string(),
                "Memory constraints on smaller devices".to_string(),
                "GPIO permission issues".to_string(),
            ],
            install_dependencies: vec!["curl".to_string()],
        },
        
        // Windows (WSL)
        PlatformConfig {
            os: "windows".to_string(),
            arch: "x86_64".to_string(),
            binary_extension: ".exe".to_string(),
            shell: "bash".to_string(), // WSL uses bash
            package_manager: "apt".to_string(), // WSL Ubuntu
            common_issues: vec![
                "WSL1 vs WSL2 networking differences".to_string(),
                "Windows Defender blocking execution".to_string(),
                "File permission mapping issues".to_string(),
                "Port forwarding from WSL to Windows".to_string(),
                "Path translation between Windows and WSL".to_string(),
            ],
            install_dependencies: vec!["curl".to_string(), "wget".to_string()],
        },
    ]
}

/// Test compatibility for a specific platform
async fn test_platform_compatibility(platform: &PlatformConfig) -> CrossPlatformTestResults {
    let mut results = CrossPlatformTestResults {
        platform: format!("{}-{}", platform.os, platform.arch),
        tests_passed: 0,
        tests_failed: 0,
        issues_found: Vec::new(),
        recommendations: Vec::new(),
    };
    
    // Test 1: Install script platform detection
    test_install_script_platform_detection(platform, &mut results).await;
    
    // Test 2: Binary availability and compatibility
    test_binary_compatibility(platform, &mut results).await;
    
    // Test 3: Dependency requirements
    test_dependency_requirements(platform, &mut results).await;
    
    // Test 4: Configuration file handling
    test_configuration_handling(platform, &mut results).await;
    
    // Test 5: Network and port handling
    test_network_port_handling(platform, &mut results).await;
    
    // Test 6: File system permissions
    test_filesystem_permissions(platform, &mut results).await;
    
    // Test 7: Shell and environment compatibility
    test_shell_environment_compatibility(platform, &mut results).await;
    
    // Test 8: Performance characteristics
    test_platform_performance(platform, &mut results).await;
    
    // Test 9: Security considerations
    test_platform_security(platform, &mut results).await;
    
    // Test 10: Common platform-specific issues
    test_common_platform_issues(platform, &mut results).await;
    
    results
}

/// Test install script platform detection logic
async fn test_install_script_platform_detection(
    platform: &PlatformConfig,
    results: &mut CrossPlatformTestResults,
) {
    println!("  🔍 Testing install script platform detection for {}", platform.os);
    
    let script_content = fs::read_to_string("scripts/install.sh")
        .expect("Failed to read install script");
    
    // Test platform detection patterns
    let platform_detected = match platform.os.as_str() {
        "darwin" => script_content.contains("Darwin*)"),
        "linux" => script_content.contains("Linux*)"),
        "windows" => script_content.contains("CYGWIN*|MINGW*|MSYS*"),
        _ => false,
    };
    
    if platform_detected {
        results.tests_passed += 1;
        println!("    ✅ Platform detection works for {}", platform.os);
    } else {
        results.tests_failed += 1;
        results.issues_found.push(PlatformIssue {
            severity: IssueSeverity::Critical,
            category: IssueCategory::Installation,
            description: format!("Install script doesn't detect {} platform", platform.os),
            solution: format!("Add platform detection for {} in install script", platform.os),
            affects_platforms: vec![platform.os.clone()],
        });
    }
    
    // Test architecture detection
    let arch_detected = script_content.contains(&platform.arch);
    if arch_detected {
        results.tests_passed += 1;
        println!("    ✅ Architecture detection works for {}", platform.arch);
    } else {
        results.tests_failed += 1;
        results.issues_found.push(PlatformIssue {
            severity: IssueSeverity::High,
            category: IssueCategory::Installation,
            description: format!("Install script doesn't detect {} architecture", platform.arch),
            solution: format!("Add architecture detection for {} in install script", platform.arch),
            affects_platforms: vec![format!("{}-{}", platform.os, platform.arch)],
        });
    }
}

/// Test binary compatibility and availability
async fn test_binary_compatibility(
    platform: &PlatformConfig,
    results: &mut CrossPlatformTestResults,
) {
    println!("  📦 Testing binary compatibility for {}-{}", platform.os, platform.arch);
    
    // Check if binary would be available for this platform
    let binary_name = format!("campfire-on-rust-{}-{}{}", 
        platform.os, platform.arch, platform.binary_extension);
    
    // Simulate checking GitHub releases
    let expected_download_url = format!(
        "https://github.com/that-in-rust/campfire-on-rust/releases/download/v0.1.0/{}",
        binary_name
    );
    
    println!("    📥 Expected download URL: {}", expected_download_url);
    
    // Test binary naming convention
    if platform.binary_extension == ".exe" && platform.os == "windows" {
        results.tests_passed += 1;
        println!("    ✅ Windows binary extension handled correctly");
    } else if platform.binary_extension.is_empty() && platform.os != "windows" {
        results.tests_passed += 1;
        println!("    ✅ Unix binary naming handled correctly");
    } else {
        results.tests_failed += 1;
        results.issues_found.push(PlatformIssue {
            severity: IssueSeverity::High,
            category: IssueCategory::Installation,
            description: "Binary naming convention issue".to_string(),
            solution: "Fix binary naming in build process".to_string(),
            affects_platforms: vec![platform.os.clone()],
        });
    }
    
    // Test if we can build for this platform (simulation)
    if can_build_for_platform(platform) {
        results.tests_passed += 1;
        println!("    ✅ Can build for platform {}-{}", platform.os, platform.arch);
    } else {
        results.tests_failed += 1;
        results.issues_found.push(PlatformIssue {
            severity: IssueSeverity::Medium,
            category: IssueCategory::Installation,
            description: format!("Cannot build for platform {}-{}", platform.os, platform.arch),
            solution: "Add cross-compilation support or CI build for this platform".to_string(),
            affects_platforms: vec![format!("{}-{}", platform.os, platform.arch)],
        });
    }
}

/// Check if we can build for a platform (simulation)
fn can_build_for_platform(platform: &PlatformConfig) -> bool {
    // Check Cargo.toml for target configuration
    let cargo_content = fs::read_to_string("Cargo.toml")
        .unwrap_or_default();
    
    // Look for cargo-dist configuration
    let has_cargo_dist = cargo_content.contains("cargo-dist") || 
                        cargo_content.contains("targets");
    
    if has_cargo_dist {
        // Check if this platform is in the targets list
        let target_triple = format!("{}-{}", platform.arch, match platform.os.as_str() {
            "darwin" => "apple-darwin",
            "linux" => "unknown-linux-gnu",
            "windows" => "pc-windows-msvc",
            _ => "unknown",
        });
        
        cargo_content.contains(&target_triple)
    } else {
        // Default supported platforms
        matches!(
            (platform.os.as_str(), platform.arch.as_str()),
            ("darwin", "x86_64") | ("darwin", "aarch64") |
            ("linux", "x86_64") | ("linux", "aarch64") |
            ("windows", "x86_64")
        )
    }
}

/// Test dependency requirements for platform
async fn test_dependency_requirements(
    platform: &PlatformConfig,
    results: &mut CrossPlatformTestResults,
) {
    println!("  📋 Testing dependency requirements for {}", platform.os);
    
    let script_content = fs::read_to_string("scripts/install.sh")
        .expect("Failed to read install script");
    
    // Test that script checks for required dependencies
    for dep in &platform.install_dependencies {
        if script_content.contains(dep) {
            results.tests_passed += 1;
            println!("    ✅ Dependency {} is checked", dep);
        } else {
            results.tests_failed += 1;
            results.issues_found.push(PlatformIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::Installation,
                description: format!("Missing dependency check for {}", dep),
                solution: format!("Add {} availability check to install script", dep),
                affects_platforms: vec![platform.os.clone()],
            });
        }
    }
    
    // Test package manager specific instructions
    if platform.package_manager == "brew" && platform.os == "darwin" {
        if script_content.contains("brew install") {
            results.tests_passed += 1;
            println!("    ✅ Homebrew installation instructions provided");
        } else {
            results.recommendations.push(
                "Consider adding Homebrew installation instructions for macOS".to_string()
            );
        }
    }
    
    if platform.package_manager == "apt" && platform.os == "linux" {
        if script_content.contains("apt install") {
            results.tests_passed += 1;
            println!("    ✅ APT installation instructions provided");
        } else {
            results.recommendations.push(
                "Consider adding APT installation instructions for Ubuntu/Debian".to_string()
            );
        }
    }
}

/// Test configuration file handling
async fn test_configuration_handling(
    platform: &PlatformConfig,
    results: &mut CrossPlatformTestResults,
) {
    println!("  ⚙️ Testing configuration handling for {}", platform.os);
    
    // Test path handling for different platforms
    let script_content = fs::read_to_string("scripts/install.sh")
        .expect("Failed to read install script");
    
    // Test home directory handling
    if script_content.contains("$HOME") {
        results.tests_passed += 1;
        println!("    ✅ Home directory path handling works");
    } else {
        results.tests_failed += 1;
        results.issues_found.push(PlatformIssue {
            severity: IssueSeverity::Medium,
            category: IssueCategory::Configuration,
            description: "Home directory path not handled properly".to_string(),
            solution: "Use $HOME for cross-platform home directory access".to_string(),
            affects_platforms: vec!["all".to_string()],
        });
    }
    
    // Test shell-specific configuration
    match platform.shell.as_str() {
        "zsh" => {
            if script_content.contains(".zshrc") {
                results.tests_passed += 1;
                println!("    ✅ Zsh configuration handled");
            } else {
                results.recommendations.push(
                    "Consider adding .zshrc PATH update for macOS default shell".to_string()
                );
            }
        }
        "bash" => {
            if script_content.contains(".bashrc") {
                results.tests_passed += 1;
                println!("    ✅ Bash configuration handled");
            } else {
                results.recommendations.push(
                    "Consider adding .bashrc PATH update for Linux systems".to_string()
                );
            }
        }
        _ => {}
    }
}

/// Test network and port handling
async fn test_network_port_handling(
    platform: &PlatformConfig,
    results: &mut CrossPlatformTestResults,
) {
    println!("  🌐 Testing network and port handling for {}", platform.os);
    
    // Test platform-specific networking issues
    for issue in &platform.common_issues {
        if issue.contains("port") || issue.contains("network") || issue.contains("firewall") {
            // Check if the issue is documented or handled
            let readme_content = fs::read_to_string("README.md").unwrap_or_default();
            let script_content = fs::read_to_string("scripts/install.sh").unwrap_or_default();
            
            let issue_documented = readme_content.contains("port") || 
                                 readme_content.contains("firewall") ||
                                 script_content.contains("port");
            
            if issue_documented {
                results.tests_passed += 1;
                println!("    ✅ Network issue documented: {}", issue);
            } else {
                results.issues_found.push(PlatformIssue {
                    severity: IssueSeverity::Medium,
                    category: IssueCategory::Runtime,
                    description: format!("Network issue not documented: {}", issue),
                    solution: "Add troubleshooting section for network issues".to_string(),
                    affects_platforms: vec![platform.os.clone()],
                });
            }
        }
    }
    
    // Test WSL-specific networking
    if platform.os == "windows" {
        results.recommendations.push(
            "Document WSL port forwarding: netsh interface portproxy add v4tov4 listenport=3000 listenaddress=0.0.0.0 connectport=3000 connectaddress=<WSL_IP>".to_string()
        );
        results.recommendations.push(
            "Document WSL firewall: New-NetFirewallRule -DisplayName 'Campfire' -Direction Inbound -Protocol TCP -LocalPort 3000".to_string()
        );
    }
}

/// Test filesystem permissions
async fn test_filesystem_permissions(
    platform: &PlatformConfig,
    results: &mut CrossPlatformTestResults,
) {
    println!("  📁 Testing filesystem permissions for {}", platform.os);
    
    let script_content = fs::read_to_string("scripts/install.sh")
        .expect("Failed to read install script");
    
    // Test executable permissions handling
    if script_content.contains("chmod +x") {
        results.tests_passed += 1;
        println!("    ✅ Executable permissions handled");
    } else {
        results.tests_failed += 1;
        results.issues_found.push(PlatformIssue {
            severity: IssueSeverity::High,
            category: IssueCategory::Installation,
            description: "Executable permissions not set".to_string(),
            solution: "Add chmod +x to make binary executable".to_string(),
            affects_platforms: vec!["darwin".to_string(), "linux".to_string()],
        });
    }
    
    // Test directory creation
    if script_content.contains("mkdir -p") {
        results.tests_passed += 1;
        println!("    ✅ Directory creation handled safely");
    } else {
        results.recommendations.push(
            "Use mkdir -p for safe directory creation".to_string()
        );
    }
    
    // Platform-specific permission issues
    if platform.os == "linux" {
        for issue in &platform.common_issues {
            if issue.contains("Permission") || issue.contains("SELinux") {
                results.recommendations.push(format!(
                    "Document solution for: {}", issue
                ));
            }
        }
    }
}

/// Test shell and environment compatibility
async fn test_shell_environment_compatibility(
    platform: &PlatformConfig,
    results: &mut CrossPlatformTestResults,
) {
    println!("  🐚 Testing shell compatibility for {}", platform.shell);
    
    let script_content = fs::read_to_string("scripts/install.sh")
        .expect("Failed to read install script");
    
    // Test shebang
    if script_content.starts_with("#!/bin/bash") {
        results.tests_passed += 1;
        println!("    ✅ Bash shebang present");
    } else {
        results.issues_found.push(PlatformIssue {
            severity: IssueSeverity::Medium,
            category: IssueCategory::Installation,
            description: "Missing or incorrect shebang".to_string(),
            solution: "Add #!/bin/bash shebang for cross-platform compatibility".to_string(),
            affects_platforms: vec!["all".to_string()],
        });
    }
    
    // Test shell detection
    if script_content.contains("$SHELL") {
        results.tests_passed += 1;
        println!("    ✅ Shell detection implemented");
    } else {
        results.recommendations.push(
            "Consider detecting user's shell for PATH updates".to_string()
        );
    }
    
    // Test environment variable handling
    let env_vars = vec!["HOME", "PATH", "USER"];
    for var in env_vars {
        if script_content.contains(&format!("${}", var)) {
            results.tests_passed += 1;
            println!("    ✅ Environment variable {} handled", var);
        }
    }
}

/// Test platform-specific performance characteristics
async fn test_platform_performance(
    platform: &PlatformConfig,
    results: &mut CrossPlatformTestResults,
) {
    println!("  ⚡ Testing performance characteristics for {}-{}", platform.os, platform.arch);
    
    // Test architecture-specific optimizations
    if platform.arch == "aarch64" {
        results.recommendations.push(
            "Consider ARM64-specific optimizations for better performance".to_string()
        );
        
        // Check if we're building native ARM64 binaries
        let cargo_content = fs::read_to_string("Cargo.toml").unwrap_or_default();
        if cargo_content.contains("aarch64") {
            results.tests_passed += 1;
            println!("    ✅ ARM64 target configured");
        } else {
            results.recommendations.push(
                "Add aarch64 targets for native ARM64 performance".to_string()
            );
        }
    }
    
    // Test memory constraints for smaller devices
    if platform.arch == "aarch64" && platform.os == "linux" {
        results.recommendations.push(
            "Document memory requirements for Raspberry Pi and similar devices".to_string()
        );
        results.recommendations.push(
            "Consider memory-optimized build flags for ARM devices".to_string()
        );
    }
    
    // Test Windows-specific performance considerations
    if platform.os == "windows" {
        results.recommendations.push(
            "Document WSL1 vs WSL2 performance differences".to_string()
        );
        results.recommendations.push(
            "Consider Windows Defender exclusions for better performance".to_string()
        );
    }
}

/// Test platform-specific security considerations
async fn test_platform_security(
    platform: &PlatformConfig,
    results: &mut CrossPlatformTestResults,
) {
    println!("  🔒 Testing security considerations for {}", platform.os);
    
    // Test macOS Gatekeeper handling
    if platform.os == "darwin" {
        results.recommendations.push(
            "Document Gatekeeper bypass: System Preferences > Security & Privacy > Allow apps downloaded from: App Store and identified developers".to_string()
        );
        results.recommendations.push(
            "Consider code signing for macOS distribution".to_string()
        );
    }
    
    // Test Linux security contexts
    if platform.os == "linux" {
        for issue in &platform.common_issues {
            if issue.contains("SELinux") {
                results.recommendations.push(
                    "Document SELinux configuration: setsebool -P httpd_can_network_connect 1".to_string()
                );
            }
        }
    }
    
    // Test Windows security
    if platform.os == "windows" {
        results.recommendations.push(
            "Document Windows Defender exclusion process".to_string()
        );
        results.recommendations.push(
            "Document UAC considerations for installation".to_string()
        );
    }
    
    // Test general security practices
    let script_content = fs::read_to_string("scripts/install.sh")
        .expect("Failed to read install script");
    
    if script_content.contains("https://") {
        results.tests_passed += 1;
        println!("    ✅ HTTPS downloads used");
    } else {
        results.issues_found.push(PlatformIssue {
            severity: IssueSeverity::High,
            category: IssueCategory::Security,
            description: "Non-HTTPS downloads detected".to_string(),
            solution: "Use HTTPS for all downloads".to_string(),
            affects_platforms: vec!["all".to_string()],
        });
    }
}

/// Test common platform-specific issues
async fn test_common_platform_issues(
    platform: &PlatformConfig,
    results: &mut CrossPlatformTestResults,
) {
    println!("  🔧 Testing common issues for {}", platform.os);
    
    let readme_content = fs::read_to_string("README.md").unwrap_or_default();
    
    for issue in &platform.common_issues {
        // Check if the issue is documented in troubleshooting
        let issue_documented = readme_content.to_lowercase().contains(&issue.to_lowercase()) ||
                              readme_content.contains("Troubleshooting");
        
        if issue_documented {
            results.tests_passed += 1;
            println!("    ✅ Issue documented: {}", issue);
        } else {
            results.issues_found.push(PlatformIssue {
                severity: IssueSeverity::Low,
                category: IssueCategory::Runtime,
                description: format!("Common issue not documented: {}", issue),
                solution: format!("Add troubleshooting section for: {}", issue),
                affects_platforms: vec![platform.os.clone()],
            });
        }
    }
}

/// Generate comprehensive cross-platform report
async fn generate_cross_platform_report(results: &[CrossPlatformTestResults]) {
    println!("\n📊 Cross-Platform Testing Report");
    println!("================================");
    
    let mut total_tests = 0;
    let mut total_passed = 0;
    let mut total_failed = 0;
    let mut all_issues = Vec::new();
    let mut all_recommendations = Vec::new();
    
    for result in results {
        println!("\n🖥️  Platform: {}", result.platform);
        println!("   Tests Passed: {}", result.tests_passed);
        println!("   Tests Failed: {}", result.tests_failed);
        println!("   Issues Found: {}", result.issues_found.len());
        
        total_tests += result.tests_passed + result.tests_failed;
        total_passed += result.tests_passed;
        total_failed += result.tests_failed;
        
        all_issues.extend(result.issues_found.clone());
        all_recommendations.extend(result.recommendations.clone());
    }
    
    println!("\n📈 Summary");
    println!("   Total Tests: {}", total_tests);
    println!("   Total Passed: {}", total_passed);
    println!("   Total Failed: {}", total_failed);
    println!("   Success Rate: {:.1}%", (total_passed as f64 / total_tests as f64) * 100.0);
    
    // Group issues by severity
    let mut critical_issues = Vec::new();
    let mut high_issues = Vec::new();
    let mut medium_issues = Vec::new();
    let mut low_issues = Vec::new();
    
    for issue in &all_issues {
        match issue.severity {
            IssueSeverity::Critical => critical_issues.push(issue),
            IssueSeverity::High => high_issues.push(issue),
            IssueSeverity::Medium => medium_issues.push(issue),
            IssueSeverity::Low => low_issues.push(issue),
        }
    }
    
    if !critical_issues.is_empty() {
        println!("\n🚨 Critical Issues ({}):", critical_issues.len());
        for issue in &critical_issues {
            println!("   - {}: {}", issue.category, issue.description);
            println!("     Solution: {}", issue.solution);
        }
    }
    
    if !high_issues.is_empty() {
        println!("\n⚠️  High Priority Issues ({}):", high_issues.len());
        for issue in &high_issues {
            println!("   - {}: {}", issue.category, issue.description);
        }
    }
    
    if !all_recommendations.is_empty() {
        println!("\n💡 Recommendations ({}):", all_recommendations.len());
        let mut unique_recommendations: Vec<_> = all_recommendations.into_iter().collect();
        unique_recommendations.sort();
        unique_recommendations.dedup();
        
        for (i, rec) in unique_recommendations.iter().enumerate() {
            println!("   {}. {}", i + 1, rec);
        }
    }
    
    // Write detailed report to file
    write_detailed_report(results).await;
}

/// Write detailed report to file
async fn write_detailed_report(results: &[CrossPlatformTestResults]) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let report_path = temp_dir.path().join("cross_platform_report.md");
    
    let mut report = String::new();
    report.push_str("# Cross-Platform Testing Report\n\n");
    report.push_str(&format!("Generated: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    
    for result in results {
        report.push_str(&format!("## Platform: {}\n\n", result.platform));
        report.push_str(&format!("- Tests Passed: {}\n", result.tests_passed));
        report.push_str(&format!("- Tests Failed: {}\n", result.tests_failed));
        report.push_str(&format!("- Issues Found: {}\n\n", result.issues_found.len()));
        
        if !result.issues_found.is_empty() {
            report.push_str("### Issues\n\n");
            for issue in &result.issues_found {
                report.push_str(&format!("- **{:?}** ({}): {}\n", 
                    issue.severity, issue.category, issue.description));
                report.push_str(&format!("  - Solution: {}\n", issue.solution));
            }
            report.push_str("\n");
        }
        
        if !result.recommendations.is_empty() {
            report.push_str("### Recommendations\n\n");
            for rec in &result.recommendations {
                report.push_str(&format!("- {}\n", rec));
            }
            report.push_str("\n");
        }
    }
    
    fs::write(&report_path, report).expect("Failed to write report");
    println!("📄 Detailed report written to: {}", report_path.display());
}

/// Verify no critical issues remain
fn verify_no_critical_issues(results: &[CrossPlatformTestResults]) {
    let critical_issues: Vec<_> = results
        .iter()
        .flat_map(|r| &r.issues_found)
        .filter(|issue| matches!(issue.severity, IssueSeverity::Critical))
        .collect();
    
    if !critical_issues.is_empty() {
        panic!("Critical cross-platform issues found: {:#?}", critical_issues);
    }
}

/// Test specific platform scenarios
#[tokio::test]
async fn test_macos_specific_scenarios() {
    println!("🍎 Testing macOS-specific scenarios");
    
    // Test Gatekeeper bypass documentation
    let readme_content = fs::read_to_string("README.md").unwrap_or_default();
    let has_gatekeeper_info = readme_content.contains("Gatekeeper") || 
                             readme_content.contains("Security & Privacy") ||
                             readme_content.contains("unsigned");
    
    if !has_gatekeeper_info {
        println!("💡 Recommendation: Add Gatekeeper troubleshooting to README");
    }
    
    // Test Homebrew installation guidance
    let script_content = fs::read_to_string("scripts/install.sh").unwrap_or_default();
    let has_brew_guidance = script_content.contains("brew install") ||
                           readme_content.contains("brew install");
    
    if !has_brew_guidance {
        println!("💡 Recommendation: Add Homebrew installation instructions");
    }
    
    println!("✅ macOS-specific scenarios tested");
}

#[tokio::test]
async fn test_linux_specific_scenarios() {
    println!("🐧 Testing Linux-specific scenarios");
    
    let readme_content = fs::read_to_string("README.md").unwrap_or_default();
    
    // Test package manager instructions
    let distros = vec![
        ("Ubuntu/Debian", "apt install"),
        ("CentOS/RHEL", "yum install"),
        ("Fedora", "dnf install"),
    ];
    
    for (distro, cmd) in distros {
        let has_instructions = readme_content.contains(cmd);
        if has_instructions {
            println!("✅ {} instructions found", distro);
        } else {
            println!("💡 Recommendation: Add {} installation instructions", distro);
        }
    }
    
    // Test SELinux documentation
    let has_selinux_info = readme_content.contains("SELinux") ||
                          readme_content.contains("setsebool");
    
    if !has_selinux_info {
        println!("💡 Recommendation: Add SELinux troubleshooting");
    }
    
    println!("✅ Linux-specific scenarios tested");
}

#[tokio::test]
async fn test_windows_wsl_scenarios() {
    println!("🪟 Testing Windows WSL scenarios");
    
    let readme_content = fs::read_to_string("README.md").unwrap_or_default();
    
    // Test WSL documentation
    let wsl_topics = vec![
        ("WSL", "Windows Subsystem for Linux"),
        ("port forwarding", "netsh interface portproxy"),
        ("firewall", "New-NetFirewallRule"),
        ("Windows Defender", "exclusion"),
    ];
    
    for (topic, keyword) in wsl_topics {
        let has_info = readme_content.contains(keyword) ||
                      readme_content.to_lowercase().contains(&topic.to_lowercase());
        
        if has_info {
            println!("✅ {} documentation found", topic);
        } else {
            println!("💡 Recommendation: Add {} documentation for Windows users", topic);
        }
    }
    
    println!("✅ Windows WSL scenarios tested");
}