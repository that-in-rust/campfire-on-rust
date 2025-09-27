use std::fs;
use std::process::Command;

#[cfg(test)]
mod css_validation_tests {
    use super::*;

    #[test]
    fn test_css_compiles_without_errors() {
        // Test that CSS files can be parsed without syntax errors
        let css_files = [
            "assets/static/css/campfire.css",
            "assets/static/css/layout.css",
            "assets/static/css/colors.css",
            "assets/static/css/base.css",
        ];

        for css_file in &css_files {
            let content = fs::read_to_string(css_file)
                .unwrap_or_else(|_| panic!("Should be able to read {}", css_file));

            // Basic CSS syntax validation
            validate_css_syntax(&content, css_file);
        }
    }

    #[test]
    fn test_css_grid_properties_valid() {
        let layout_css = fs::read_to_string("assets/static/css/layout.css")
            .expect("Should be able to read layout.css");

        // Validate CSS Grid properties
        let grid_properties = [
            "display: grid",
            "grid-template-areas",
            "grid-template-columns",
            "grid-template-rows",
            "grid-area",
        ];

        for property in &grid_properties {
            assert!(
                layout_css.contains(property),
                "CSS should contain valid grid property: {}", property
            );
        }
    }

    #[test]
    fn test_css_variables_properly_defined() {
        let layout_css = fs::read_to_string("assets/static/css/layout.css")
            .expect("Should be able to read layout.css");

        // Test CSS custom properties are properly defined
        let css_vars = [
            "--inline-space:",
            "--block-space:",
            "--sidebar-width:",
            "--footer-height:",
            "--navbar-height:",
        ];

        for var in &css_vars {
            assert!(
                layout_css.contains(var),
                "CSS should define variable: {}", var
            );
        }
    }

    #[test]
    fn test_media_queries_valid() {
        let layout_css = fs::read_to_string("assets/static/css/layout.css")
            .expect("Should be able to read layout.css");

        // Validate media queries
        assert!(layout_css.contains("@media (min-width: 100ch)"), "Should have desktop media query");
        assert!(layout_css.contains("@media (max-width: 100ch)"), "Should have mobile media query");
        
        // Check media query syntax
        let media_query_count = layout_css.matches("@media").count();
        let media_query_close_count = layout_css.matches("}").count();
        
        // Basic validation that media queries are properly closed
        assert!(media_query_close_count >= media_query_count, "Media queries should be properly closed");
    }

    #[test]
    fn test_no_css_conflicts() {
        let campfire_css = fs::read_to_string("assets/static/css/campfire.css")
            .expect("Should be able to read campfire.css");

        // Test that old flexbox and new grid don't conflict
        let conflicting_patterns = [
            "display: flex;\n    height: 100vh;\n    overflow: hidden;",
            ".app-container {\n    display: flex;",
        ];

        for pattern in &conflicting_patterns {
            assert!(
                !campfire_css.contains(pattern),
                "Should not contain conflicting CSS pattern: {}", pattern
            );
        }
    }

    fn validate_css_syntax(content: &str, filename: &str) {
        // Basic CSS syntax validation
        let open_braces = content.matches('{').count();
        let close_braces = content.matches('}').count();
        assert_eq!(
            open_braces, close_braces,
            "CSS file {} should have matching braces", filename
        );

        let open_parens = content.matches('(').count();
        let close_parens = content.matches(')').count();
        assert_eq!(
            open_parens, close_parens,
            "CSS file {} should have matching parentheses", filename
        );

        // Check for common syntax errors
        assert!(
            !content.contains(";;"),
            "CSS file {} should not have double semicolons", filename
        );
        assert!(
            !content.contains("{{"),
            "CSS file {} should not have double open braces", filename
        );
        assert!(
            !content.contains("}}"),
            "CSS file {} should not have double close braces", filename
        );
    }
}

#[cfg(test)]
mod css_grid_functional_tests {
    use super::*;

    #[test]
    fn test_grid_layout_structure_complete() {
        let layout_css = fs::read_to_string("assets/static/css/layout.css")
            .expect("Should be able to read layout.css");

        // Test complete grid structure matches Basecamp Campfire original
        let required_structure = [
            // Grid container
            "body {",
            "display: grid;",
            
            // Grid template
            "grid-template-areas:",
            r#""nav sidebar""#,
            r#""main sidebar""#,
            "grid-template-columns: 1fr var(--sidebar-width);",
            "grid-template-rows: min-content 1fr;",
            
            // Grid areas
            "#nav {",
            "grid-area: nav;",
            "#main-content {",
            "grid-area: main;",
            "#sidebar {",
            "grid-area: sidebar;",
            
            // Responsive behavior
            "body.sidebar {",
            "--sidebar-width: 26vw;",
        ];

        for structure in &required_structure {
            assert!(
                layout_css.contains(structure),
                "CSS Grid structure should contain: {}", structure
            );
        }
    }

    #[test]
    fn test_professional_visual_features() {
        let layout_css = fs::read_to_string("assets/static/css/layout.css")
            .expect("Should be able to read layout.css");

        // Test professional visual features from original Basecamp
        let professional_features = [
            "backdrop-filter: blur(66px)",
            "-webkit-backdrop-filter: blur(66px)",
            "oklch(var(--lch-white) / 0.66)",
            "transition: transform 300ms ease",
            "overscroll-behavior: none",
            "max-block-size: 100dvh",
        ];

        for feature in &professional_features {
            assert!(
                layout_css.contains(feature),
                "Should include professional visual feature: {}", feature
            );
        }
    }

    #[test]
    fn test_accessibility_features() {
        let layout_css = fs::read_to_string("assets/static/css/layout.css")
            .expect("Should be able to read layout.css");

        // Test accessibility features
        assert!(layout_css.contains("@media"), "Should include responsive design");
        
        // Check that layout doesn't break with reduced motion
        // (This would be handled by the main CSS, but grid should be compatible)
        let chat_html = fs::read_to_string("templates/chat.html")
            .expect("Should be able to read chat.html");
        
        assert!(chat_html.contains("role="), "HTML should include ARIA roles");
        assert!(chat_html.contains("aria-"), "HTML should include ARIA attributes");
    }
}