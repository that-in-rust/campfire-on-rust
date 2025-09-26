// L1 Core Testing Framework Integration Tests
// Following TDD-First Architecture: STUB → RED → GREEN → REFACTOR

use campfire_on_rust::testing::{
    CICDTestingFramework, ProductionCICDTesting, TestFrameworkError, TestingHealth,
    cargo_dist::{MockCargoDistProvider, CargoDistProvider},
    MockCriterionProvider, MockProptestProvider,
    CriterionProvider, ProptestProvider,
};
use std::time::Duration;

/// Integration test for L1 Core Testing Framework
/// This test follows the GREEN phase - now implemented with all providers
#[tokio::test]
async fn test_l1_core_testing_framework_integration() {
    // ARRANGE: Create production CI/CD testing framework with all providers
    let cargo_dist = Box::new(MockCargoDistProvider::new(true));
    let criterion = Box::new(MockCriterionProvider::new(true));
    let proptest = Box::new(MockProptestProvider::new(true));
    
    let framework = ProductionCICDTesting::new(cargo_dist, criterion, proptest);
    
    // ACT: Generate comprehensive testing report
    let report = framework.generate_testing_report().await.unwrap();
    
    // ASSERT: Verify all testing components work together
    assert!(matches!(report.overall_health, TestingHealth::Excellent));
    assert!(report.build_report.success_rate > 0.95);
    assert!(report.performance_report.violations.is_empty());
    assert_eq!(report.property_report.failed_cases, 0);
    
    // ASSERT: All L1 components are integrated
    assert!(!report.build_report.platforms.is_empty());
    assert!(!report.performance_report.benchmarks.is_empty());
    assert!(!report.property_report.properties.is_empty());
}

/// Test cross-platform build validation
#[tokio::test]
async fn test_cross_platform_build_validation() {
    // ARRANGE: Create cargo-dist provider
    let provider = MockCargoDistProvider::new(true);
    
    // ACT: Validate builds
    let report = provider.validate_builds().await.unwrap();
    
    // ASSERT: Verify build success
    assert_eq!(report.success_rate, 1.0);
    assert!(!report.platforms.is_empty());
    assert!(report.platforms.iter().all(|p| p.success));
    assert!(report.total_build_time > Duration::ZERO);
}

/// Test build failure handling
#[tokio::test]
async fn test_build_failure_handling() {
    // ARRANGE: Create failing cargo-dist provider
    let provider = MockCargoDistProvider::new(false);
    
    // ACT: Attempt to validate builds
    let result = provider.validate_builds().await;
    
    // ASSERT: Verify proper error handling
    assert!(result.is_err());
    match result.unwrap_err() {
        TestFrameworkError::CrossPlatformBuildFailed { platform, error } => {
            assert_eq!(platform, "test");
            assert!(!error.is_empty());
        }
        _ => panic!("Expected CrossPlatformBuildFailed error"),
    }
}

/// Test performance contract validation (GREEN phase - now implemented)
#[tokio::test]
async fn test_performance_contract_validation() {
    // ARRANGE: Create criterion provider
    let provider = MockCriterionProvider::new(true);
    
    // ACT: Run benchmarks with contracts
    let report = provider.run_benchmarks_with_contracts().await.unwrap();
    
    // ASSERT: Verify performance contracts
    assert!(report.violations.is_empty());
    assert!(report.overall_score > 0.8);
    assert!(!report.benchmarks.is_empty());
    assert!(!report.contracts.is_empty());
}

/// Test property-based testing (GREEN phase - now implemented)
#[tokio::test]
async fn test_property_based_testing() {
    // ARRANGE: Create proptest provider
    let provider = MockProptestProvider::new(true);
    
    // ACT: Test installation invariants
    let report = provider.test_installation_invariants().await.unwrap();
    
    // ASSERT: Verify property tests
    assert_eq!(report.failed_cases, 0);
    assert!(report.total_cases > 0);
    assert!(!report.properties.is_empty());
    assert!(report.shrunk_cases.is_empty()); // No failures, so no shrinking
}

/// Test comprehensive reporting (GREEN phase - now implemented)
#[tokio::test]
async fn test_comprehensive_reporting() {
    // ARRANGE: Create full framework with all providers
    let cargo_dist = Box::new(MockCargoDistProvider::new(true));
    let criterion = Box::new(MockCriterionProvider::new(true));
    let proptest = Box::new(MockProptestProvider::new(true));
    
    let framework = ProductionCICDTesting::new(cargo_dist, criterion, proptest);
    
    // ACT: Generate comprehensive report
    let report = framework.generate_testing_report().await.unwrap();
    
    // ASSERT: Comprehensive report includes all components
    assert!(matches!(report.overall_health, TestingHealth::Excellent));
    
    // Build report validation
    assert!(report.build_report.success_rate > 0.95);
    assert!(!report.build_report.platforms.is_empty());
    
    // Performance report validation
    assert!(report.performance_report.violations.is_empty());
    assert!(!report.performance_report.benchmarks.is_empty());
    assert!(report.performance_report.overall_score > 0.8);
    
    // Property report validation
    assert_eq!(report.property_report.failed_cases, 0);
    assert!(report.property_report.total_cases > 0);
    assert!(!report.property_report.properties.is_empty());
    
    // Recommendations should be minimal for excellent health
    assert!(report.recommendations.len() <= 2);
}