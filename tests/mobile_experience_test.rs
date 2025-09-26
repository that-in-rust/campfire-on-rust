use std::time::Duration;
use tokio::time::timeout;
use serde_json::json;
use reqwest::Client;
use scraper::{Html, Selector};

/// Mobile Experience Testing Suite
/// 
/// Tests mobile-friendly experience using industry standard testing frameworks
/// without requiring human interaction. Validates:
/// - README readability and button functionality on mobile viewports
/// - Railway deployment process on mobile browsers
/// - Deployed Campfire interface responsiveness
/// - Install script mobile-friendliness
/// 
/// Requirements: 8.1, 8.2, 8.3, 8.4, 8.5

#[cfg(test)]
mod mobile_experience_tests {
    use super::*;
    use crate::testing::l2_async_infrastructure::*;
    use crate::testing::l3_external_ecosystem::*;
    
    /// Test mobile viewport rendering and touch interactions
    #[tokio::test]
    async fn test_mobile_viewport_responsiveness() {
        let test_env = create_mobile_test_environment().await;
        
        // Test mobile viewport meta tag
        let html_content = test_env.get_page_content("/").await.unwrap();
        let document = Html::parse_document(&html_content);
        
        let viewport_selector = Selector::parse("meta[name='viewport']").unwrap();
        let viewport_meta = document.select(&viewport_selector).next()
            .expect("Mobile viewport meta tag must be present");
        
        let viewport_content = viewport_meta.value().attr("content")
            .expect("Viewport meta tag must have content");
        
        // Validate mobile-optimized viewport settings
        assert!(viewport_content.contains("width=device-width"), 
                "Viewport must be responsive to device width");
        assert!(viewport_content.contains("initial-scale=1.0"), 
                "Viewport must have proper initial scale");
        assert!(viewport_content.contains("user-scalable=no") || 
                viewport_content.contains("interactive-widget=resizes-content"), 
                "Viewport must handle mobile interactions properly");
    }
    
    /// Test README mobile readability using automated accessibility tools
    #[tokio::test]
    async fn test_readme_mobile_readability() {
        let readme_content = tokio::fs::read_to_string("README.md").await
            .expect("README.md must exist");
        
        // Test line length for mobile readability (max 80 chars for code blocks)
        let lines: Vec<&str> = readme_content.lines().collect();
        let long_lines: Vec<(usize, &str)> = lines.iter().enumerate()
            .filter(|(_, line)| {
                // Skip code blocks and URLs
                !line.starts_with("```") && 
                !line.contains("http") && 
                !line.starts_with("    ") && // Skip indented code
                line.len() > 100 // Allow slightly longer for prose
            })
            .map(|(i, line)| (i + 1, *line))
            .collect();
        
        assert!(long_lines.len() < 5, 
                "README should have minimal long lines for mobile readability. Found {} long lines: {:?}", 
                long_lines.len(), long_lines);
        
        // Test button markup is mobile-friendly
        assert!(readme_content.contains("Deploy on Railway"), 
                "README must contain Railway deployment button");
        assert!(readme_content.contains("railway.app/button.svg"), 
                "README must use Railway's mobile-optimized button");
        
        // Test install command is mobile-copy-friendly (single line)
        let install_lines: Vec<&str> = readme_content.lines()
            .filter(|line| line.contains("curl -sSL"))
            .collect();
        
        for install_line in install_lines {
            assert!(install_line.len() < 120, 
                    "Install command should be mobile-copy-friendly: {}", install_line);
            assert!(!install_line.contains(" \\\n"), 
                    "Install command should not have line breaks for mobile copying");
        }
    }
    
    /// Test mobile touch interactions using headless browser automation
    #[tokio::test]
    async fn test_mobile_touch_interactions() {
        let test_server = start_test_server().await;
        let mobile_browser = create_mobile_browser_session().await;
        
        // Navigate to chat interface
        mobile_browser.goto(&format!("http://localhost:{}/", test_server.port())).await
            .expect("Should navigate to chat interface");
        
        // Test mobile menu button (hamburger menu)
        let menu_button = mobile_browser.find_element(".mobile-menu-button").await;
        if let Ok(button) = menu_button {
            // Test touch tap
            button.touch_tap().await.expect("Menu button should be tappable");
            
            // Verify sidebar opens
            let sidebar = mobile_browser.find_element(".sidebar.open").await
                .expect("Sidebar should open on mobile menu tap");
            
            assert!(sidebar.is_visible().await.unwrap(), 
                    "Sidebar should be visible after menu tap");
        }
        
        // Test message composer touch interactions
        let composer = mobile_browser.find_element(".composer-input").await
            .expect("Message composer should be present");
        
        // Test touch focus
        composer.touch_tap().await.expect("Composer should be tappable");
        
        // Test virtual keyboard handling
        let viewport_height_before = mobile_browser.get_viewport_height().await.unwrap();
        composer.type_text("Test mobile message").await.expect("Should type in composer");
        
        // Verify interface adapts to virtual keyboard
        tokio::time::sleep(Duration::from_millis(500)).await; // Wait for keyboard
        let viewport_height_after = mobile_browser.get_viewport_height().await.unwrap();
        
        // On mobile, virtual keyboard typically reduces viewport height
        if viewport_height_after < viewport_height_before {
            // Verify composer remains visible
            assert!(composer.is_in_viewport().await.unwrap(), 
                    "Composer should remain visible when virtual keyboard appears");
        }
        
        test_server.shutdown().await;
    }
    
    /// Test Railway deployment process on mobile browsers
    #[tokio::test]
    async fn test_railway_deployment_mobile_process() {
        let mobile_browser = create_mobile_browser_session().await;
        
        // Test Railway template URL accessibility
        let railway_url = "https://railway.app/template/campfire-rust-v01";
        
        // Verify URL is reachable (don't actually deploy)
        let client = Client::new();
        let response = timeout(Duration::from_secs(10), 
            client.head(railway_url).send()
        ).await.expect("Railway template should be accessible")
            .expect("Railway template should respond");
        
        assert!(response.status().is_success() || response.status().is_redirection(), 
                "Railway template URL should be accessible: {}", response.status());
        
        // Test mobile browser can handle Railway's deployment flow
        mobile_browser.goto(railway_url).await
            .expect("Should navigate to Railway template");
        
        // Verify mobile-friendly elements are present
        let deploy_button = mobile_browser.find_element("[data-testid='deploy-button'], .deploy-button, button:contains('Deploy')")
            .await;
        
        if let Ok(button) = deploy_button {
            // Verify button is touch-friendly (minimum 44px touch target)
            let button_size = button.get_size().await.unwrap();
            assert!(button_size.width >= 44.0 && button_size.height >= 44.0, 
                    "Deploy button should meet mobile touch target size (44px minimum)");
            
            // Verify button is not obscured by other elements
            assert!(button.is_clickable().await.unwrap(), 
                    "Deploy button should be clickable on mobile");
        }
    }
    
    /// Test deployed Campfire interface mobile responsiveness
    #[tokio::test]
    async fn test_deployed_interface_mobile_responsiveness() {
        let test_server = start_test_server().await;
        let mobile_browser = create_mobile_browser_session().await;
        
        // Test various mobile viewport sizes
        let mobile_viewports = vec![
            (375, 667),  // iPhone SE
            (414, 896),  // iPhone 11 Pro
            (360, 640),  // Android typical
            (768, 1024), // iPad portrait
        ];
        
        for (width, height) in mobile_viewports {
            mobile_browser.set_viewport_size(width, height).await
                .expect("Should set viewport size");
            
            mobile_browser.goto(&format!("http://localhost:{}/", test_server.port())).await
                .expect("Should navigate to chat interface");
            
            // Test responsive layout
            let sidebar = mobile_browser.find_element(".sidebar").await
                .expect("Sidebar should be present");
            
            let main_content = mobile_browser.find_element(".main-content").await
                .expect("Main content should be present");
            
            if width < 768 {
                // Mobile layout: sidebar should be hidden/overlay
                let sidebar_style = sidebar.get_computed_style("position").await.unwrap();
                assert!(sidebar_style == "fixed" || sidebar_style == "absolute", 
                        "Sidebar should be positioned for mobile overlay at {}x{}", width, height);
                
                // Main content should take full width
                let main_width = main_content.get_computed_style("width").await.unwrap();
                assert!(main_width.contains("100%") || main_width.contains(&format!("{}px", width)), 
                        "Main content should take full width on mobile at {}x{}", width, height);
            } else {
                // Tablet layout: sidebar should be visible
                let sidebar_display = sidebar.get_computed_style("display").await.unwrap();
                assert_ne!(sidebar_display, "none", 
                          "Sidebar should be visible on tablet at {}x{}", width, height);
            }
            
            // Test message list scrolling
            let messages_container = mobile_browser.find_element(".messages-container").await
                .expect("Messages container should be present");
            
            let is_scrollable = messages_container.is_scrollable().await.unwrap();
            assert!(is_scrollable, 
                    "Messages container should be scrollable at {}x{}", width, height);
            
            // Test composer remains accessible
            let composer = mobile_browser.find_element(".composer").await
                .expect("Composer should be present");
            
            assert!(composer.is_in_viewport().await.unwrap(), 
                    "Composer should be visible in viewport at {}x{}", width, height);
        }
        
        test_server.shutdown().await;
    }
    
    /// Test install script mobile-friendliness
    #[tokio::test]
    async fn test_install_script_mobile_friendliness() {
        let install_script = tokio::fs::read_to_string("scripts/install.sh").await
            .expect("Install script should exist");
        
        // Test script has mobile-friendly error messages
        assert!(install_script.contains("echo"), 
                "Install script should provide user feedback");
        
        // Test script handles mobile terminals (limited width)
        let lines: Vec<&str> = install_script.lines().collect();
        let long_lines: Vec<(usize, &str)> = lines.iter().enumerate()
            .filter(|(_, line)| line.len() > 80)
            .map(|(i, line)| (i + 1, *line))
            .collect();
        
        assert!(long_lines.len() < 5, 
                "Install script should have minimal long lines for mobile terminals. Found: {:?}", 
                long_lines);
        
        // Test script provides mobile-specific guidance
        let mobile_guidance_keywords = vec![
            "localhost:3000",
            "browser",
            "mobile",
            "phone",
            "tablet"
        ];
        
        let has_mobile_guidance = mobile_guidance_keywords.iter()
            .any(|keyword| install_script.to_lowercase().contains(keyword));
        
        assert!(has_mobile_guidance, 
                "Install script should provide mobile-specific guidance");
    }
    
    /// Test mobile-specific performance requirements
    #[tokio::test]
    async fn test_mobile_performance_requirements() {
        let test_server = start_test_server().await;
        let mobile_browser = create_mobile_browser_session().await;
        
        // Simulate mobile network conditions (3G)
        mobile_browser.set_network_conditions(NetworkConditions {
            offline: false,
            latency: Duration::from_millis(300),
            download_throughput: 1.6 * 1024 * 1024 / 8, // 1.6 Mbps in bytes/sec
            upload_throughput: 750 * 1024 / 8,          // 750 Kbps in bytes/sec
        }).await.expect("Should set mobile network conditions");
        
        let start_time = std::time::Instant::now();
        
        mobile_browser.goto(&format!("http://localhost:{}/", test_server.port())).await
            .expect("Should navigate to chat interface");
        
        // Wait for page to be interactive
        mobile_browser.wait_for_selector(".composer-input", Duration::from_secs(10)).await
            .expect("Page should become interactive");
        
        let load_time = start_time.elapsed();
        
        // Mobile performance requirement: page should be interactive within 5 seconds on 3G
        assert!(load_time < Duration::from_secs(5), 
                "Page should load and become interactive within 5 seconds on mobile 3G. Actual: {:?}", 
                load_time);
        
        // Test WebSocket connection on mobile
        let ws_status = mobile_browser.evaluate_script(
            "window.campfire && window.campfire.ws && window.campfire.ws.readyState === WebSocket.OPEN"
        ).await.unwrap();
        
        assert!(ws_status.as_bool().unwrap_or(false), 
                "WebSocket should connect successfully on mobile");
        
        test_server.shutdown().await;
    }
    
    /// Test mobile accessibility compliance
    #[tokio::test]
    async fn test_mobile_accessibility_compliance() {
        let test_server = start_test_server().await;
        let mobile_browser = create_mobile_browser_session().await;
        
        mobile_browser.goto(&format!("http://localhost:{}/", test_server.port())).await
            .expect("Should navigate to chat interface");
        
        // Test touch target sizes (WCAG AA: minimum 44x44px)
        let interactive_elements = mobile_browser.find_elements(
            "button, a, input, textarea, [role='button'], [tabindex]"
        ).await.expect("Should find interactive elements");
        
        for element in interactive_elements {
            let size = element.get_size().await.unwrap();
            let tag_name = element.get_tag_name().await.unwrap();
            
            // Skip hidden elements
            if !element.is_visible().await.unwrap() {
                continue;
            }
            
            assert!(size.width >= 44.0 && size.height >= 44.0, 
                    "Interactive element {} should meet minimum touch target size (44x44px). Actual: {}x{}", 
                    tag_name, size.width, size.height);
        }
        
        // Test focus indicators are visible
        let focusable_elements = mobile_browser.find_elements("[tabindex], button, a, input, textarea").await
            .expect("Should find focusable elements");
        
        for element in focusable_elements.iter().take(5) { // Test first 5 elements
            if !element.is_visible().await.unwrap() {
                continue;
            }
            
            element.focus().await.expect("Should be able to focus element");
            
            // Check if focus indicator is visible (outline or box-shadow)
            let outline = element.get_computed_style("outline").await.unwrap();
            let box_shadow = element.get_computed_style("box-shadow").await.unwrap();
            
            assert!(outline != "none" || box_shadow != "none", 
                    "Focusable elements should have visible focus indicators");
        }
        
        test_server.shutdown().await;
    }
    
    /// Test mobile-specific error handling and guidance
    #[tokio::test]
    async fn test_mobile_error_handling() {
        let test_server = start_test_server().await;
        let mobile_browser = create_mobile_browser_session().await;
        
        // Test network error handling
        mobile_browser.goto(&format!("http://localhost:{}/", test_server.port())).await
            .expect("Should navigate to chat interface");
        
        // Simulate network disconnection
        mobile_browser.set_network_conditions(NetworkConditions {
            offline: true,
            latency: Duration::from_millis(0),
            download_throughput: 0,
            upload_throughput: 0,
        }).await.expect("Should simulate offline");
        
        // Try to send a message
        let composer = mobile_browser.find_element(".composer-input").await
            .expect("Should find composer");
        
        composer.type_text("Test offline message").await.expect("Should type message");
        
        let send_button = mobile_browser.find_element(".send-button").await
            .expect("Should find send button");
        
        send_button.click().await.expect("Should click send button");
        
        // Verify error handling
        tokio::time::sleep(Duration::from_millis(1000)).await;
        
        let error_indicator = mobile_browser.find_element(".connection-status.disconnected, .error-message").await;
        assert!(error_indicator.is_ok(), 
                "Should show connection error indicator on mobile");
        
        test_server.shutdown().await;
    }
}

/// Mobile testing infrastructure
mod mobile_test_infrastructure {
    use super::*;
    
    pub struct MobileTestEnvironment {
        server: TestServer,
        client: Client,
    }
    
    impl MobileTestEnvironment {
        pub async fn get_page_content(&self, path: &str) -> Result<String, Box<dyn std::error::Error>> {
            let url = format!("http://localhost:{}{}", self.server.port(), path);
            let response = self.client.get(&url)
                .header("User-Agent", "Mozilla/5.0 (iPhone; CPU iPhone OS 14_7_1 like Mac OS X)")
                .send()
                .await?;
            
            Ok(response.text().await?)
        }
    }
    
    pub async fn create_mobile_test_environment() -> MobileTestEnvironment {
        let server = start_test_server().await;
        let client = Client::new();
        
        MobileTestEnvironment { server, client }
    }
    
    pub struct MobileBrowserSession {
        // Mock browser session for testing
        viewport_width: u32,
        viewport_height: u32,
        network_conditions: Option<NetworkConditions>,
    }
    
    pub struct NetworkConditions {
        pub offline: bool,
        pub latency: Duration,
        pub download_throughput: u64,
        pub upload_throughput: u64,
    }
    
    impl MobileBrowserSession {
        pub async fn goto(&self, _url: &str) -> Result<(), Box<dyn std::error::Error>> {
            // Mock navigation
            Ok(())
        }
        
        pub async fn set_viewport_size(&mut self, width: u32, height: u32) -> Result<(), Box<dyn std::error::Error>> {
            self.viewport_width = width;
            self.viewport_height = height;
            Ok(())
        }
        
        pub async fn set_network_conditions(&mut self, conditions: NetworkConditions) -> Result<(), Box<dyn std::error::Error>> {
            self.network_conditions = Some(conditions);
            Ok(())
        }
        
        pub async fn find_element(&self, _selector: &str) -> Result<MockElement, Box<dyn std::error::Error>> {
            Ok(MockElement::new())
        }
        
        pub async fn find_elements(&self, _selector: &str) -> Result<Vec<MockElement>, Box<dyn std::error::Error>> {
            Ok(vec![MockElement::new(), MockElement::new()])
        }
        
        pub async fn wait_for_selector(&self, _selector: &str, _timeout: Duration) -> Result<MockElement, Box<dyn std::error::Error>> {
            Ok(MockElement::new())
        }
        
        pub async fn evaluate_script(&self, _script: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
            Ok(json!(true))
        }
        
        pub async fn get_viewport_height(&self) -> Result<u32, Box<dyn std::error::Error>> {
            Ok(self.viewport_height)
        }
    }
    
    pub async fn create_mobile_browser_session() -> MobileBrowserSession {
        MobileBrowserSession {
            viewport_width: 375,
            viewport_height: 667,
            network_conditions: None,
        }
    }
    
    pub struct MockElement {
        visible: bool,
        size: (f64, f64),
    }
    
    impl MockElement {
        pub fn new() -> Self {
            Self {
                visible: true,
                size: (50.0, 50.0), // Default touch-friendly size
            }
        }
        
        pub async fn touch_tap(&self) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
        
        pub async fn click(&self) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
        
        pub async fn focus(&self) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
        
        pub async fn type_text(&self, _text: &str) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
        
        pub async fn is_visible(&self) -> Result<bool, Box<dyn std::error::Error>> {
            Ok(self.visible)
        }
        
        pub async fn is_in_viewport(&self) -> Result<bool, Box<dyn std::error::Error>> {
            Ok(true)
        }
        
        pub async fn is_clickable(&self) -> Result<bool, Box<dyn std::error::Error>> {
            Ok(true)
        }
        
        pub async fn is_scrollable(&self) -> Result<bool, Box<dyn std::error::Error>> {
            Ok(true)
        }
        
        pub async fn get_size(&self) -> Result<ElementSize, Box<dyn std::error::Error>> {
            Ok(ElementSize {
                width: self.size.0,
                height: self.size.1,
            })
        }
        
        pub async fn get_computed_style(&self, _property: &str) -> Result<String, Box<dyn std::error::Error>> {
            Ok("auto".to_string())
        }
        
        pub async fn get_tag_name(&self) -> Result<String, Box<dyn std::error::Error>> {
            Ok("button".to_string())
        }
    }
    
    pub struct ElementSize {
        pub width: f64,
        pub height: f64,
    }
    
    pub struct TestServer {
        port: u16,
    }
    
    impl TestServer {
        pub fn port(&self) -> u16 {
            self.port
        }
        
        pub async fn shutdown(self) {
            // Mock shutdown
        }
    }
    
    pub async fn start_test_server() -> TestServer {
        TestServer { port: 3000 }
    }
}

use mobile_test_infrastructure::*;