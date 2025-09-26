use std::collections::HashMap;
use regex::Regex;

/// Mobile CSS Validation Test Suite
/// 
/// Validates CSS for mobile-friendly design patterns using automated analysis.
/// Tests responsive design, touch targets, and mobile-specific optimizations.
/// 
/// Requirements: 8.1, 8.2, 8.3, 8.4

#[cfg(test)]
mod mobile_css_validation_tests {
    use super::*;
    
    /// Test CSS contains proper mobile viewport and responsive design
    #[tokio::test]
    async fn test_css_mobile_responsiveness() {
        let css_content = tokio::fs::read_to_string("assets/static/css/campfire.css").await
            .expect("CSS file should exist");
        
        // Test for mobile-first responsive design patterns
        assert!(css_content.contains("@media"), 
                "CSS should contain media queries for responsive design");
        
        // Test for mobile breakpoints
        let mobile_breakpoint_regex = Regex::new(r"@media\s*\([^)]*max-width:\s*768px[^)]*\)").unwrap();
        assert!(mobile_breakpoint_regex.is_match(&css_content), 
                "CSS should contain mobile breakpoint (max-width: 768px)");
        
        // Test for touch-friendly sizing
        let touch_target_patterns = vec![
            r"min-height:\s*44px",
            r"min-width:\s*44px", 
            r"padding:\s*[0-9]+px\s+[0-9]+px", // Adequate padding for touch
        ];
        
        let has_touch_targets = touch_target_patterns.iter()
            .any(|pattern| Regex::new(pattern).unwrap().is_match(&css_content));
        
        assert!(has_touch_targets, 
                "CSS should include touch-friendly target sizes (minimum 44px)");
        
        // Test for mobile-specific optimizations
        let mobile_optimizations = vec![
            "user-select", // Prevent text selection on touch
            "touch-action", // Control touch gestures
            "-webkit-tap-highlight-color", // Remove tap highlights
            "overflow-x", // Handle horizontal scrolling
        ];
        
        let optimization_count = mobile_optimizations.iter()
            .filter(|opt| css_content.contains(*opt))
            .count();
        
        assert!(optimization_count >= 2, 
                "CSS should include mobile-specific optimizations. Found: {}/{}",
                optimization_count, mobile_optimizations.len());
    }
    
    /// Test CSS handles mobile layout patterns correctly
    #[tokio::test]
    async fn test_mobile_layout_patterns() {
        let css_content = tokio::fs::read_to_string("assets/static/css/campfire.css").await
            .expect("CSS file should exist");
        
        // Test for flexible layout patterns
        let layout_patterns = vec![
            (r"display:\s*flex", "Flexbox for flexible layouts"),
            (r"flex-direction:\s*column", "Column layout for mobile stacking"),
            (r"width:\s*100%", "Full width utilization"),
            (r"max-width:\s*100%", "Prevent horizontal overflow"),
        ];
        
        for (pattern, description) in layout_patterns {
            let regex = Regex::new(pattern).unwrap();
            assert!(regex.is_match(&css_content), 
                    "CSS should include {}: {}", description, pattern);
        }
        
        // Test for mobile navigation patterns
        let mobile_nav_patterns = vec![
            r"position:\s*fixed", // Fixed positioning for mobile headers/navs
            r"z-index:\s*[0-9]+", // Proper layering
            r"transform:\s*translate", // Hardware-accelerated animations
        ];
        
        let nav_pattern_count = mobile_nav_patterns.iter()
            .filter(|pattern| Regex::new(pattern).unwrap().is_match(&css_content))
            .count();
        
        assert!(nav_pattern_count >= 2, 
                "CSS should include mobile navigation patterns. Found: {}/{}",
                nav_pattern_count, mobile_nav_patterns.len());
    }
    
    /// Test CSS performance optimizations for mobile
    #[tokio::test]
    async fn test_mobile_performance_optimizations() {
        let css_content = tokio::fs::read_to_string("assets/static/css/campfire.css").await
            .expect("CSS file should exist");
        
        // Test for hardware acceleration hints
        let performance_patterns = vec![
            r"transform:\s*translate3d", // Force hardware acceleration
            r"will-change:", // Hint for optimization
            r"backface-visibility:\s*hidden", // Prevent flickering
        ];
        
        let perf_pattern_count = performance_patterns.iter()
            .filter(|pattern| Regex::new(pattern).unwrap().is_match(&css_content))
            .count();
        
        // At least one performance optimization should be present
        if perf_pattern_count == 0 {
            println!("Warning: No explicit mobile performance optimizations found in CSS");
        }
        
        // Test for efficient selectors (avoid complex nesting)
        let lines: Vec<&str> = css_content.lines().collect();
        let complex_selectors: Vec<&str> = lines.iter()
            .filter(|line| {
                let selector_complexity = line.matches(' ').count() + 
                                        line.matches('>').count() + 
                                        line.matches('+').count() + 
                                        line.matches('~').count();
                selector_complexity > 4 // Avoid overly complex selectors
            })
            .cloned()
            .collect();
        
        assert!(complex_selectors.len() < 5, 
                "CSS should avoid overly complex selectors for mobile performance. Found: {:?}",
                complex_selectors);
        
        // Test file size is reasonable for mobile
        let css_size = css_content.len();
        assert!(css_size < 100_000, // 100KB limit for mobile
                "CSS file should be under 100KB for mobile performance. Current size: {} bytes",
                css_size);
    }
    
    /// Test CSS accessibility for mobile users
    #[tokio::test]
    async fn test_mobile_accessibility_css() {
        let css_content = tokio::fs::read_to_string("assets/static/css/campfire.css").await
            .expect("CSS file should exist");
        
        // Test for focus indicators
        let focus_patterns = vec![
            r":focus\s*{[^}]*outline",
            r":focus\s*{[^}]*box-shadow",
            r":focus\s*{[^}]*border",
        ];
        
        let has_focus_indicators = focus_patterns.iter()
            .any(|pattern| Regex::new(pattern).unwrap().is_match(&css_content));
        
        assert!(has_focus_indicators, 
                "CSS should include visible focus indicators for accessibility");
        
        // Test for high contrast support
        let contrast_patterns = vec![
            r"@media\s*\([^)]*prefers-contrast:\s*high[^)]*\)",
            r"@media\s*\([^)]*prefers-color-scheme:\s*dark[^)]*\)",
        ];
        
        let has_contrast_support = contrast_patterns.iter()
            .any(|pattern| Regex::new(pattern).unwrap().is_match(&css_content));
        
        assert!(has_contrast_support, 
                "CSS should support high contrast and dark mode preferences");
        
        // Test for reduced motion support
        let motion_pattern = Regex::new(r"@media\s*\([^)]*prefers-reduced-motion[^)]*\)").unwrap();
        assert!(motion_pattern.is_match(&css_content), 
                "CSS should respect reduced motion preferences");
        
        // Test color contrast ratios (basic check for light/dark values)
        let color_regex = Regex::new(r"color:\s*#([0-9a-fA-F]{3,6})").unwrap();
        let background_regex = Regex::new(r"background(?:-color)?:\s*#([0-9a-fA-F]{3,6})").unwrap();
        
        let colors: Vec<&str> = color_regex.find_iter(&css_content)
            .map(|m| m.as_str())
            .collect();
        
        let backgrounds: Vec<&str> = background_regex.find_iter(&css_content)
            .map(|m| m.as_str())
            .collect();
        
        // Basic check: should have both light and dark colors for contrast
        assert!(!colors.is_empty() && !backgrounds.is_empty(), 
                "CSS should define both text colors and background colors for proper contrast");
    }
    
    /// Test CSS handles mobile-specific input types
    #[tokio::test]
    async fn test_mobile_input_handling() {
        let css_content = tokio::fs::read_to_string("assets/static/css/campfire.css").await
            .expect("CSS file should exist");
        
        // Test for input styling that works on mobile
        let input_patterns = vec![
            r"input\s*\[[^]]*type[^]]*\]", // Type-specific input styling
            r"textarea", // Textarea styling
            r"::placeholder", // Placeholder styling
            r"::-webkit-input-placeholder", // WebKit placeholder
        ];
        
        let input_pattern_count = input_patterns.iter()
            .filter(|pattern| Regex::new(pattern).unwrap().is_match(&css_content))
            .count();
        
        assert!(input_pattern_count >= 2, 
                "CSS should include mobile-friendly input styling. Found: {}/{}",
                input_pattern_count, input_patterns.len());
        
        // Test for mobile keyboard handling
        let keyboard_patterns = vec![
            r"resize:\s*none", // Prevent textarea resize on mobile
            r"appearance:\s*none", // Remove default styling
            r"-webkit-appearance:\s*none", // WebKit appearance
        ];
        
        let keyboard_pattern_count = keyboard_patterns.iter()
            .filter(|pattern| Regex::new(pattern).unwrap().is_match(&css_content))
            .count();
        
        if keyboard_pattern_count == 0 {
            println!("Warning: No explicit mobile keyboard handling found in CSS");
        }
    }
    
    /// Test CSS grid and flexbox usage for mobile layouts
    #[tokio::test]
    async fn test_modern_layout_techniques() {
        let css_content = tokio::fs::read_to_string("assets/static/css/campfire.css").await
            .expect("CSS file should exist");
        
        // Test for modern layout techniques
        let modern_layout_patterns = vec![
            (r"display:\s*flex", "Flexbox"),
            (r"display:\s*grid", "CSS Grid"),
            (r"flex-wrap:\s*wrap", "Flex wrapping"),
            (r"gap:\s*[0-9]+", "Gap property"),
        ];
        
        let mut found_patterns = Vec::new();
        for (pattern, name) in modern_layout_patterns {
            if Regex::new(pattern).unwrap().is_match(&css_content) {
                found_patterns.push(name);
            }
        }
        
        assert!(!found_patterns.is_empty(), 
                "CSS should use modern layout techniques for mobile responsiveness");
        
        println!("Found modern layout techniques: {:?}", found_patterns);
        
        // Test for mobile-first approach (min-width media queries)
        let mobile_first_regex = Regex::new(r"@media\s*\([^)]*min-width[^)]*\)").unwrap();
        let desktop_first_regex = Regex::new(r"@media\s*\([^)]*max-width[^)]*\)").unwrap();
        
        let mobile_first_count = mobile_first_regex.find_iter(&css_content).count();
        let desktop_first_count = desktop_first_regex.find_iter(&css_content).count();
        
        // Prefer mobile-first approach, but allow both
        if mobile_first_count == 0 && desktop_first_count > 0 {
            println!("Note: CSS uses desktop-first approach. Consider mobile-first for better performance.");
        }
    }
    
    /// Test CSS custom properties (CSS variables) for theming
    #[tokio::test]
    async fn test_css_custom_properties() {
        let css_content = tokio::fs::read_to_string("assets/static/css/campfire.css").await
            .expect("CSS file should exist");
        
        // Test for CSS custom properties
        let custom_property_regex = Regex::new(r"--[a-zA-Z-]+:\s*[^;]+;").unwrap();
        let custom_properties: Vec<&str> = custom_property_regex.find_iter(&css_content)
            .map(|m| m.as_str())
            .collect();
        
        assert!(!custom_properties.is_empty(), 
                "CSS should use custom properties for maintainable theming");
        
        // Test for var() usage
        let var_usage_regex = Regex::new(r"var\(--[a-zA-Z-]+\)").unwrap();
        let var_usages: Vec<&str> = var_usage_regex.find_iter(&css_content)
            .map(|m| m.as_str())
            .collect();
        
        assert!(!var_usages.is_empty(), 
                "CSS should use var() to reference custom properties");
        
        // Test for common theming properties
        let theming_properties = vec![
            "--color-primary",
            "--color-background", 
            "--color-text",
            "--font-family",
            "--border-radius",
        ];
        
        let found_theming = theming_properties.iter()
            .filter(|prop| css_content.contains(*prop))
            .count();
        
        assert!(found_theming >= 3, 
                "CSS should define common theming custom properties. Found: {}/{}",
                found_theming, theming_properties.len());
    }
}

/// CSS Analysis Utilities
mod css_analysis {
    use super::*;
    
    pub struct CSSAnalyzer {
        content: String,
    }
    
    impl CSSAnalyzer {
        pub fn new(content: String) -> Self {
            Self { content }
        }
        
        pub fn extract_media_queries(&self) -> Vec<String> {
            let media_regex = Regex::new(r"@media[^{]+\{[^{}]*(?:\{[^{}]*\}[^{}]*)*\}").unwrap();
            media_regex.find_iter(&self.content)
                .map(|m| m.as_str().to_string())
                .collect()
        }
        
        pub fn extract_selectors(&self) -> Vec<String> {
            let selector_regex = Regex::new(r"([^{}]+)\s*\{").unwrap();
            selector_regex.captures_iter(&self.content)
                .map(|cap| cap[1].trim().to_string())
                .filter(|s| !s.starts_with('@')) // Skip at-rules
                .collect()
        }
        
        pub fn calculate_specificity(&self, selector: &str) -> u32 {
            let id_count = selector.matches('#').count() as u32;
            let class_count = (selector.matches('.').count() + 
                              selector.matches('[').count() + 
                              selector.matches(':').count()) as u32;
            let element_count = selector.split_whitespace()
                .filter(|part| !part.starts_with('.') && 
                              !part.starts_with('#') && 
                              !part.starts_with('[') &&
                              !part.starts_with(':'))
                .count() as u32;
            
            id_count * 100 + class_count * 10 + element_count
        }
        
        pub fn find_mobile_breakpoints(&self) -> Vec<u32> {
            let breakpoint_regex = Regex::new(r"(?:max-width|min-width):\s*(\d+)px").unwrap();
            breakpoint_regex.captures_iter(&self.content)
                .map(|cap| cap[1].parse::<u32>().unwrap_or(0))
                .collect()
        }
        
        pub fn analyze_performance_impact(&self) -> PerformanceAnalysis {
            let file_size = self.content.len();
            let selector_count = self.extract_selectors().len();
            let media_query_count = self.extract_media_queries().len();
            
            let complex_selectors = self.extract_selectors().iter()
                .filter(|selector| self.calculate_specificity(selector) > 100)
                .count();
            
            PerformanceAnalysis {
                file_size,
                selector_count,
                media_query_count,
                complex_selectors,
                performance_score: self.calculate_performance_score(
                    file_size, selector_count, complex_selectors
                ),
            }
        }
        
        fn calculate_performance_score(&self, file_size: usize, selector_count: usize, complex_selectors: usize) -> u32 {
            let mut score = 100;
            
            // Penalize large file size
            if file_size > 50_000 { score -= 20; }
            if file_size > 100_000 { score -= 30; }
            
            // Penalize too many selectors
            if selector_count > 500 { score -= 15; }
            if selector_count > 1000 { score -= 25; }
            
            // Penalize complex selectors
            if complex_selectors > 50 { score -= 10; }
            if complex_selectors > 100 { score -= 20; }
            
            score.max(0) as u32
        }
    }
    
    pub struct PerformanceAnalysis {
        pub file_size: usize,
        pub selector_count: usize,
        pub media_query_count: usize,
        pub complex_selectors: usize,
        pub performance_score: u32,
    }
}

use css_analysis::*;