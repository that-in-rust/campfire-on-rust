use std::fs;
use std::path::Path;

#[cfg(test)]
mod css_grid_layout_tests {
    use super::*;

    #[test]
    fn test_layout_css_exists_and_valid() {
        let layout_css_path = "assets/static/css/layout.css";
        assert!(
            Path::new(layout_css_path).exists(),
            "layout.css file should exist at {}", layout_css_path
        );

        let content = fs::read_to_string(layout_css_path)
            .expect("Should be able to read layout.css");
        
        // Verify CSS Grid implementation
        assert!(content.contains("display: grid"), "Should use CSS Grid");
        assert!(content.contains("grid-template-areas"), "Should define grid template areas");
        assert!(content.contains("grid-template-columns"), "Should define grid columns");
        assert!(content.contains("grid-template-rows"), "Should define grid rows");
    }

    #[test]
    fn test_css_grid_template_areas_correct() {
        let layout_css = fs::read_to_string("assets/static/css/layout.css")
            .expect("Should be able to read layout.css");

        // Verify the correct grid template areas from original Basecamp Campfire
        assert!(layout_css.contains(r#""nav sidebar""#), "Should have nav sidebar area");
        assert!(layout_css.contains(r#""main sidebar""#), "Should have main sidebar area");
    }

    #[test]
    fn test_spacing_variables_defined() {
        let layout_css = fs::read_to_string("assets/static/css/layout.css")
            .expect("Should be able to read layout.css");

        // Verify spacing variables from original utilities.css
        assert!(layout_css.contains("--inline-space: 1ch"), "Should define inline-space");
        assert!(layout_css.contains("--block-space: 1rem"), "Should define block-space");
        assert!(layout_css.contains("--inline-space-half"), "Should define inline-space-half");
        assert!(layout_css.contains("--block-space-half"), "Should define block-space-half");
    }

    #[test]
    fn test_responsive_sidebar_width() {
        let layout_css = fs::read_to_string("assets/static/css/layout.css")
            .expect("Should be able to read layout.css");

        // Verify responsive sidebar width system
        assert!(layout_css.contains("--sidebar-width: 0vw"), "Should default sidebar width to 0");
        assert!(layout_css.contains("--sidebar-width: 26vw"), "Should set sidebar width to 26vw on large screens");
        assert!(layout_css.contains("@media (min-width: 100ch)"), "Should use 100ch breakpoint");
    }

    #[test]
    fn test_grid_areas_assigned() {
        let layout_css = fs::read_to_string("assets/static/css/layout.css")
            .expect("Should be able to read layout.css");

        // Verify grid areas are properly assigned
        assert!(layout_css.contains("grid-area: nav"), "Should assign nav grid area");
        assert!(layout_css.contains("grid-area: main"), "Should assign main grid area");
        assert!(layout_css.contains("grid-area: sidebar"), "Should assign sidebar grid area");
    }

    #[test]
    fn test_backdrop_filter_implementation() {
        let layout_css = fs::read_to_string("assets/static/css/layout.css")
            .expect("Should be able to read layout.css");

        // Verify backdrop filter for professional appearance
        assert!(layout_css.contains("backdrop-filter: blur"), "Should use backdrop filter");
        assert!(layout_css.contains("-webkit-backdrop-filter"), "Should include webkit prefix");
    }

    #[test]
    fn test_campfire_css_imports_layout() {
        let campfire_css = fs::read_to_string("assets/static/css/campfire.css")
            .expect("Should be able to read campfire.css");

        assert!(
            campfire_css.contains("@import url('./layout.css')"),
            "campfire.css should import layout.css"
        );
    }

    #[test]
    fn test_flexbox_layout_removed() {
        let campfire_css = fs::read_to_string("assets/static/css/campfire.css")
            .expect("Should be able to read campfire.css");

        // Verify old flexbox layout is removed/replaced
        let flexbox_patterns = [
            ".app-container {\n    display: flex;",
            ".sidebar {\n    width: 280px;\n    background-color: var(--color-surface);\n    border-right: 1px solid var(--color-border);\n    display: flex;",
            ".main-content {\n    flex: 1;\n    display: flex;"
        ];

        for pattern in &flexbox_patterns {
            assert!(
                !campfire_css.contains(pattern),
                "Old flexbox pattern should be removed: {}", pattern
            );
        }
    }

    #[test]
    fn test_css_syntax_validity() {
        let layout_css = fs::read_to_string("assets/static/css/layout.css")
            .expect("Should be able to read layout.css");

        // Basic CSS syntax validation
        let open_braces = layout_css.matches('{').count();
        let close_braces = layout_css.matches('}').count();
        assert_eq!(open_braces, close_braces, "CSS should have matching braces");

        // Check for common syntax errors
        assert!(!layout_css.contains(";;"), "Should not have double semicolons");
        assert!(!layout_css.contains("{{"), "Should not have double open braces");
        assert!(!layout_css.contains("}}"), "Should not have double close braces");
    }

    #[test]
    fn test_html_template_updated() {
        let chat_html = fs::read_to_string("templates/chat.html")
            .expect("Should be able to read chat.html");

        // Verify HTML structure matches CSS Grid requirements
        assert!(chat_html.contains(r#"<body class="sidebar">"#), "Body should have sidebar class");
        assert!(chat_html.contains(r#"id="nav""#), "Should have nav element with ID");
        assert!(chat_html.contains(r#"id="sidebar""#), "Should have sidebar element with ID");
        assert!(chat_html.contains(r#"id="main-content""#), "Should have main-content element with ID");
    }

    #[test]
    fn test_legacy_compatibility() {
        let layout_css = fs::read_to_string("assets/static/css/layout.css")
            .expect("Should be able to read layout.css");

        // Verify legacy class compatibility is maintained
        assert!(layout_css.contains(".app-container"), "Should maintain .app-container compatibility");
        assert!(layout_css.contains(".main-content"), "Should maintain .main-content compatibility");
        assert!(layout_css.contains(".sidebar"), "Should maintain .sidebar compatibility");
    }
}

#[cfg(test)]
mod css_grid_integration_tests {
    use super::*;

    #[test]
    fn test_complete_css_grid_implementation() {
        // This test verifies the complete CSS Grid implementation matches the original Basecamp Campfire

        let layout_css = fs::read_to_string("assets/static/css/layout.css")
            .expect("Should be able to read layout.css");

        // Test the complete grid structure
        let required_patterns = vec![
            "display: grid",
            "grid-template-areas:",
            r#""nav sidebar""#,
            r#""main sidebar""#,
            "grid-template-columns: 1fr var(--sidebar-width)",
            "grid-template-rows: min-content 1fr",
            "max-block-size: 100dvh",
            "--sidebar-width: 0vw",
            "--sidebar-width: 26vw",
            "grid-area: nav",
            "grid-area: main", 
            "grid-area: sidebar",
            "backdrop-filter: blur(66px)",
            "oklch(var(--lch-white) / 0.66)"
        ];

        for pattern in required_patterns {
            assert!(
                layout_css.contains(pattern),
                "CSS Grid implementation should contain: {}", pattern
            );
        }
    }

    #[test]
    fn test_responsive_behavior() {
        let layout_css = fs::read_to_string("assets/static/css/layout.css")
            .expect("Should be able to read layout.css");

        // Test responsive grid behavior
        assert!(layout_css.contains("@media (min-width: 100ch)"), "Should have desktop breakpoint");
        assert!(layout_css.contains("@media (max-width: 100ch)"), "Should have mobile breakpoint");
        assert!(layout_css.contains("transform: translate(100%)"), "Should hide sidebar on mobile");
        assert!(layout_css.contains("transform: translate(0)"), "Should show sidebar when open");
    }

    #[test]
    fn test_performance_optimizations() {
        let layout_css = fs::read_to_string("assets/static/css/layout.css")
            .expect("Should be able to read layout.css");

        // Test performance optimizations
        assert!(layout_css.contains("transition: transform 300ms ease"), "Should have smooth transitions");
        assert!(layout_css.contains("overscroll-behavior: none"), "Should prevent overscroll");
        assert!(layout_css.contains("contain: inline-size") || layout_css.contains("overflow: auto"), "Should contain layout");
    }
}