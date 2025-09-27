use std::fs;

#[tokio::test]
async fn test_lch_color_system_implementation() {
    // Test that colors.css exists and contains LCH color definitions
    let colors_css = fs::read_to_string("assets/static/css/colors.css")
        .expect("colors.css should exist");
    
    // Verify LCH color values match original Basecamp Campfire exactly
    assert!(colors_css.contains("--lch-black: 0% 0 0;"));
    assert!(colors_css.contains("--lch-white: 100% 0 0;"));
    assert!(colors_css.contains("--lch-gray: 96% 0.005 96;"));
    assert!(colors_css.contains("--lch-blue: 54% 0.23 255;"));
    assert!(colors_css.contains("--lch-red: 51% 0.2 31;"));
    assert!(colors_css.contains("--lch-green: 65.59% 0.234 142.49;"));
    
    // Verify semantic abstractions use oklch() function
    assert!(colors_css.contains("--color-bg: oklch(var(--lch-white));"));
    assert!(colors_css.contains("--color-text: oklch(var(--lch-black));"));
    assert!(colors_css.contains("--color-link: oklch(var(--lch-blue));"));
    assert!(colors_css.contains("--color-negative: oklch(var(--lch-red));"));
    assert!(colors_css.contains("--color-positive: oklch(var(--lch-green));"));
    
    // Verify dark mode transformations exist
    assert!(colors_css.contains("@media (prefers-color-scheme: dark)"));
    assert!(colors_css.contains("--lch-black: 100% 0 0;")); // Dark mode inversion
    assert!(colors_css.contains("--lch-white: 0% 0 0;")); // Dark mode inversion
}

#[tokio::test]
async fn test_base_css_typography_system() {
    // Test that base.css exists and contains proper typography
    let base_css = fs::read_to_string("assets/static/css/base.css")
        .expect("base.css should exist");
    
    // Verify font stack includes Aptos and emoji fonts
    assert!(base_css.contains("Aptos"));
    assert!(base_css.contains("Apple Color Emoji"));
    assert!(base_css.contains("Segoe UI Emoji"));
    
    // Verify sophisticated hover system
    assert!(base_css.contains("--hover-color: var(--color-border-darker);"));
    assert!(base_css.contains("--hover-size: 0.15em;"));
    assert!(base_css.contains("box-shadow: 0 0 0 var(--hover-size) var(--hover-color);"));
    
    // Verify focus management
    assert!(base_css.contains("focus-visible"));
    assert!(base_css.contains("--outline-size: min(0.2em, 2px);"));
}

#[tokio::test]
async fn test_modular_css_architecture() {
    // Test that main CSS file imports modular files
    let main_css = fs::read_to_string("assets/static/css/campfire.css")
        .expect("campfire.css should exist");
    
    // Verify CSS imports
    assert!(main_css.contains("@import url('./colors.css');"));
    assert!(main_css.contains("@import url('./base.css');"));
    
    // Verify legacy color mappings for compatibility
    assert!(main_css.contains("--color-primary: var(--color-link);"));
    assert!(main_css.contains("--color-background: var(--color-bg);"));
    
    // Verify old RGB-based dark mode is removed
    assert!(!main_css.contains("--color-background: #1f1f1f;"));
    assert!(!main_css.contains("--color-surface: #2d2d2d;"));
}

#[tokio::test]
async fn test_color_accuracy_against_original() {
    let colors_css = fs::read_to_string("assets/static/css/colors.css")
        .expect("colors.css should exist");
    
    // Test exact LCH values match original Basecamp Campfire
    let expected_colors = vec![
        ("--lch-black", "0% 0 0"),
        ("--lch-white", "100% 0 0"),
        ("--lch-gray", "96% 0.005 96"),
        ("--lch-gray-dark", "92% 0.005 96"),
        ("--lch-gray-darker", "75% 0.005 96"),
        ("--lch-blue", "54% 0.23 255"),
        ("--lch-blue-light", "95% 0.03 255"),
        ("--lch-blue-dark", "80% 0.08 255"),
        ("--lch-orange", "70% 0.2 44"),
        ("--lch-red", "51% 0.2 31"),
        ("--lch-green", "65.59% 0.234 142.49"),
    ];
    
    for (color_name, expected_value) in expected_colors {
        let pattern = format!("{}: {};", color_name, expected_value);
        assert!(colors_css.contains(&pattern), 
            "Color {} should have exact value {}", color_name, expected_value);
    }
}

#[tokio::test]
async fn test_dark_mode_transformations() {
    let colors_css = fs::read_to_string("assets/static/css/colors.css")
        .expect("colors.css should exist");
    
    // Verify dark mode section exists
    assert!(colors_css.contains("@media (prefers-color-scheme: dark)"));
    
    // Test specific dark mode transformations match original
    let dark_mode_colors = vec![
        ("--lch-black", "100% 0 0"), // Inverted
        ("--lch-white", "0% 0 0"), // Inverted
        ("--lch-gray", "25.2% 0 0"),
        ("--lch-blue", "72.25% 0.16 248"),
        ("--lch-red", "73.8% 0.184 29.18"),
        ("--lch-green", "75% 0.21 141.89"),
    ];
    
    for (color_name, expected_value) in dark_mode_colors {
        let pattern = format!("{}: {};", color_name, expected_value);
        assert!(colors_css.contains(&pattern), 
            "Dark mode color {} should have exact value {}", color_name, expected_value);
    }
}

#[tokio::test]
async fn test_semantic_color_abstractions_completeness() {
    let colors_css = fs::read_to_string("assets/static/css/colors.css")
        .expect("colors.css should exist");
    
    // Test core semantic colors exist and use oklch()
    let core_semantic_colors = vec![
        "--color-negative: oklch(var(--lch-red));",
        "--color-positive: oklch(var(--lch-green));",
        "--color-bg: oklch(var(--lch-white));",
        "--color-message-bg: oklch(var(--lch-gray));",
        "--color-text: oklch(var(--lch-black));",
        "--color-text-reversed: oklch(var(--lch-white));",
        "--color-link: oklch(var(--lch-blue));",
        "--color-border: oklch(var(--lch-gray));",
        "--color-border-dark: oklch(var(--lch-gray-dark));",
        "--color-border-darker: oklch(var(--lch-gray-darker));",
        "--color-selected: oklch(var(--lch-blue-light));",
        "--color-selected-dark: oklch(var(--lch-blue-dark));",
        "--color-alert: oklch(var(--lch-orange));",
    ];
    
    for color_definition in core_semantic_colors {
        assert!(colors_css.contains(color_definition), 
            "Missing core semantic color: {}", color_definition);
    }
    
    // Test extended semantic abstractions for UI parity
    let extended_semantic_colors = vec![
        "--message-background: var(--color-message-bg);",
        "--message-color: var(--color-text);",
        "--btn-background: var(--color-text-reversed);",
        "--btn-color: var(--color-text);",
        "--btn-border-color: var(--color-border-darker);",
        "--hover-color: var(--color-border-darker);",
        "--link-color: var(--color-link);",
        "--outline-color: var(--color-link);",
        "--fieldset-border-color: var(--color-border);",
        "--border-color: var(--color-border);",
        "--boost-border-color: var(--color-border-dark);",
    ];
    
    for color_definition in extended_semantic_colors {
        assert!(colors_css.contains(color_definition), 
            "Missing extended semantic color: {}", color_definition);
    }
}