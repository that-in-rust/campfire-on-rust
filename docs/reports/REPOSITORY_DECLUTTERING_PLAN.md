# Repository Decluttering Plan

## Current State Analysis

**Total Files**: 426 (246 text, 180 binary)  
**Key Issues Identified**:

1. **Duplicate/Redundant Documentation**: Multiple README files, duplicate reports
2. **Development Artifacts**: Extensive .kiro dev artifacts that should be cleaned
3. **Excessive Test Files**: 60+ test files with potential overlap
4. **Script Proliferation**: 30+ scripts with potential redundancy
5. **Binary Assets**: Large sound/image files that could be optimized
6. **IDE Files**: .idea, .vscode, .DS_Store files should be gitignored

## Decluttering Strategy

### Phase 1: Remove Development Artifacts
- Clean up .kiro/dev-artifacts (keeping only essential steering files)
- Remove duplicate documentation files
- Clean up IDE-specific files

### Phase 2: Consolidate Documentation
- Keep only essential documentation files
- Remove duplicate README files
- Organize remaining docs into clear structure

### Phase 3: Optimize Test Suite
- Consolidate overlapping test files
- Remove redundant test implementations
- Keep only essential end-to-end and unit tests

### Phase 4: Script Cleanup
- Remove redundant scripts
- Keep only essential deployment and validation scripts
- Consolidate similar functionality

### Phase 5: Asset Optimization
- Review binary assets for necessity
- Optimize large files where possible
- Ensure all assets are actually used

## Files to Remove/Consolidate

### Immediate Removal (Development Artifacts)
- `.DS_Store` files
- `.idea/` directory (IDE specific)
- `.vscode/settings.json` (IDE specific)
- `.kiro/dev-artifacts/` (development artifacts)

### Documentation Consolidation
- Remove: `README_UPDATED.md` (duplicate)
- Remove: `END_TO_END_TESTING_IMPLEMENTATION_SUMMARY.md` (covered in other docs)
- Remove: `INSTALLATION_VERIFICATION_REPORT.md` (covered in GTM report)
- Remove: `PERFORMANCE_VALIDATION.md` (covered in other reports)
- Remove: `PERFORMANCE_VALIDATION_SUMMARY.md` (duplicate)
- Remove: `RAILWAY_DEPLOYMENT_TESTING.md` (covered in GTM report)
- Remove: `requirements.md` (duplicate of spec requirements)

### Test File Consolidation
- Keep: Core GTM tests, end-to-end integration tests
- Remove: Duplicate test implementations
- Remove: Development/experimental test files

### Script Cleanup
- Keep: Essential deployment and validation scripts
- Remove: Redundant testing scripts
- Remove: Development/experimental scripts

## Target State

**Estimated Final Count**: ~200 files (reduction of ~50%)
**Key Improvements**:
- Clean, professional repository structure
- Clear documentation hierarchy
- Consolidated test suite
- Essential scripts only
- Optimized for public launch