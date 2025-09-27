# Implementation Plan: Shreyas Doshi campfire-on-rust GTM

## Phase 1: Make It Work (Foundation Tasks)

- [x] 1. Fix Compilation Errors
  - Fix all 41 compilation errors preventing `cargo run` from working
  - Add missing enum variants (`TypingIndicator`, `TooManyConnections`)
  - Resolve type mismatches (u64 vs usize, Arc<T> vs T)
  - Add missing trait implementations (Clone, Debug, Serialize, Deserialize)
  - Fix lifetime issues in cache service and other modules
  - Ensure `cargo check` passes with 0 errors
  - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 9.7, 9.8_

- [x] 2. Create GitHub Release with Pre-built Binaries
  - Build optimized binaries for macOS (x86_64, aarch64), Linux (x86_64, aarch64), Windows (x86_64)
  - Create GitHub release v0.1.0 with all platform binaries
  - Test binary downloads work from GitHub releases API
  - Verify install script can download and execute binaries correctly
  - _Requirements: 10.2, 10.3, 10.4, 10.6_

- [x] 3. Implement Professional CI/CD Testing Architecture
  - Replace custom bash scripts with industry-standard testing frameworks
  - Implement L1→L2→L3 layered testing approach following TDD-First Architecture Principles
  - Create executable specifications with measurable performance contracts
  - Ensure all testing follows STUB → RED → GREEN → REFACTOR cycle
  - _Requirements: 12.1, 12.2, 12.3, 12.4, 12.5, 12.6, 12.7_

- [x] 3.1 Implement L1 Core Testing Framework (Rust Native)
  - Create trait-based CI/CD testing interfaces with dependency injection for testability
  - Implement cargo-dist configuration for professional cross-platform builds
  - Write criterion benchmarks with performance contracts for all timing claims
  - Add proptest property-based tests for installation and deployment invariants
  - _Requirements: 12.2, 12.7_

- [x] 3.2 Implement L2 Standard Library Testing (Async + Infrastructure)
  - Create testcontainers-rs integration tests for clean environment simulation
  - Implement tokio-test async testing patterns for GitHub Actions workflow validation
  - Add mockall trait-based mocking for external service dependencies
  - Write tempfile-based filesystem testing for installation script validation
  - _Requirements: 12.3, 12.6_

- [x] 3.3 Implement L3 External Ecosystem Testing (Professional Tools)
  - Configure act for local GitHub Actions workflow testing before deployment
  - Implement goss server validation tests for binary functionality verification
  - Create structured bats tests to replace custom bash script validation
  - Add docker-compose integration environments for end-to-end testing
  - _Requirements: 12.1, 12.4, 12.5_

- [x] 3.4 Create Executable Testing Specifications with Performance Contracts
  - Write WHEN...THEN...SHALL acceptance criteria tests for all CI/CD functionality
  - Implement automated regression detection with criterion benchmark baselines
  - Create structured error hierarchies using thiserror for all testing failures
  - Add comprehensive test coverage reporting with automated quality gates
  - _Requirements: 12.7, 12.5_

- [x] 3.5 Remove Custom Bash Scripts and Implement Professional Alternatives
  - Replace scripts/test-github-release.sh with cargo-dist + act integration tests
  - Replace scripts/test-install-simulation.sh with testcontainers-rs infrastructure tests
  - Replace scripts/verify-release-setup.sh with goss server validation tests
  - Implement one-command verification using professional testing frameworks
  - _Requirements: 12.5, 12.1_

- [x] 3. Verify End-to-End Installation Flow
  - Test `curl -sSL https://raw.githubusercontent.com/that-in-rust/campfire-on-rust/main/scripts/install.sh | bash` on clean machines
  - Verify application starts successfully and is accessible at http://localhost:3000
  - Test basic functionality (admin setup, create room, send message)
  - Confirm demo mode works with `CAMPFIRE_DEMO_MODE=true`
  - Test installation on macOS, Linux, and Windows (WSL)
  - _Requirements: 10.1, 10.5, 10.7, 9.1, 9.2_

- [x] 4. Test Railway Deployment End-to-End
  - Deploy using Railway template to verify it works completely
  - Test deployment completes within 3 minutes as promised
  - Verify deployed instance is accessible and functional
  - Test admin account creation and basic team chat functionality
  - Ensure deployment handles failures gracefully with clear error messages
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6_

## Phase 2: Make It Clear (GTM Implementation)

- [x] 5. Implement Two-Path README Design
  - Create clear "Try it locally" and "Deploy for your team" sections in README
  - Design simple, prominent buttons/sections for each path
  - Ensure local path shows: `curl -sSL https://raw.githubusercontent.com/that-in-rust/campfire-on-rust/main/scripts/install.sh | bash`
  - Ensure deploy path shows: Railway button → working team chat
  - Make both paths equally prominent, not one primary/one secondary
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

- [x] 6. Enhance Demo Mode for Better Local Sampling
  - Improve existing demo mode to show more realistic team chat scenarios
  - Add multiple pre-configured users with different roles and conversation styles
  - Ensure demo data demonstrates all core features (rooms, messages, search, @mentions, sounds)
  - Add clear "This is Demo Data" indicators in the interface
  - Include prominent "Deploy for Your Team" call-to-action in demo interface
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

- [x] 7. Add Simple Success Tracking
  - Add basic Google Analytics to track "Deploy Now" button clicks from README
  - Track successful Railway deployments vs failures
  - Add simple event tracking for install script downloads
  - Focus on deployment success metrics, not complex user behavior
  - Use privacy-friendly analytics approach
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

- [x] 8. Validate Performance Claims in README
  - Measure actual startup time, memory usage, and performance metrics on standard hardware
  - Update README with verified performance numbers only
  - Remove any unsubstantiated claims about speed or efficiency
  - Add benchmarking tests to validate ongoing performance claims
  - Be honest about MVP limitations and what's not yet implemented
  - _Requirements: 4.1, 4.2, 4.3, 6.1, 6.2, 6.3_

- [x] 9. Create Simple Help and Troubleshooting
  - Add troubleshooting section to README for common installation issues
  - Document Railway deployment failure scenarios and solutions
  - Provide clear error messages in install script with helpful guidance
  - Add real contact information for support (GitHub Issues, Discussions)
  - Focus help on getting deployment working, not complex diagnostics
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

- [x] 10. Ensure Mobile-Friendly Experience - using industry standard testing frameworks which do not need any human interactions to validate the experience
  - Test README readability and button functionality on mobile devices
  - Verify Railway deployment process works smoothly on mobile browsers
  - Test deployed campfire-on-rust interface responsiveness on various mobile devices
  - Ensure install script instructions are mobile-friendly
  - Add mobile-specific guidance if needed
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_

## Phase 3: Validation and Launch

- [ ] 11. End-to-End Testing on current machine (mac) using industry standard testing frameworks so you do not need any human tests. + for other platforms which you do not have right now like linux or windows - research deeply and check for what mistakes can happen
  - Test complete "Try it locally" flow: `curl | bash` → localhost:3000 on clean machines
  - Test complete "Deploy for your team" flow from GitHub README to working chat
  - Verify both paths lead to working campfire-on-rust within promised timeframes (2-3 minutes)
  - Test install script on macOS (Intel/Apple Silicon), Linux (Ubuntu/CentOS), Windows (WSL)
  - Document any platform-specific issues and provide solutions
  - _Requirements: 1.5, 2.1, 3.2, 10.1, 10.5, 10.7_

- [x] 12. Launch Preparation and Final Validation
  - Final review of README for clarity, accuracy, and mobile-friendliness - start by thanking DHH and Jason Fried for sharing campfire - and tell the story of how this is just campfire-on-rust - a humble attempt at same experience with Rust flavour
  - Ensure all links, commands, and deployment buttons work as documented
  - Verify Railway template deploys successfully in multiple regions
  - Test that support channels (GitHub Issues, Discussions) are properly configured
  - Create simple monitoring dashboard for deployment health and success rates
  - _Requirements: 5.1, 5.2, 7.3, 8.1, 8.2, 8.3_

- [x] 13. Create GitHub Release with Pre-built Binaries
  - Build optimized release binaries for all supported platforms (macOS x86_64/aarch64, Linux x86_64/aarch64, Windows x86_64)
  - Create GitHub release v0.1.0 with comprehensive release notes
  - Upload all platform binaries to GitHub release
  - Test binary downloads work from install script
  - Verify install script can download and execute binaries correctly
  - _Requirements: 10.2, 10.3, 10.4, 10.6_

- [x] 14. Enable GitHub Discussions and Community Features
  - Enable GitHub Discussions in repository settings
  - _Requirements: 7.3, 8.1_

- [x] 15. Repository Decluttering and Organization
  - Analyze repository structure using tree-with-wc.sh
  - Remove unnecessary files, outdated scripts, and development artifacts
  - Organize documentation into clear structure
  - Clean up test files and consolidate testing approach
  - Ensure clean, professional repository structure for public launch
  - _Requirements: 8.1, 8.2, 8.3_

## Success Criteria

### Phase 1 Success (Make It Work)
- [x] CHECK `cargo check` passes with 0 errors +[ ] Check GitHub releases exist with pre-built binaries for all major platforms + [ ] Check `curl | bash` install script downloads and runs successfully +[ ] `cargo run` starts application successfully and is accessible at localhost:3000 + [ ] Railway deployment template works end-to-end

### Phase 2 Success (Make It Clear)
- [x] CHECK README clearly shows two paths: local sampling and team deployment+ [ ] Demo mode provides compelling local sampling experience + [ ] Performance claims in README are verified and accurate + [ ] Simple analytics track deployment success rates +[ ] Mobile experience is functional and user-friendly

### Phase 3 Success (Validation and Launch)
- [x] CHECK End-to-end testing passes on macOS, Linux, and Windows
  -  [x] Both installation paths complete within promised timeframes (2-3 minutes) + 


  - [x] All links, commands, and deployment buttons work as documented

  - [x] Product is ready for public GTM launch with confidence


- [x] check if all the campfire on the repo - readmme - all scripts and other files is campfire-on-rust remove italics everyone - just let it be campfire-on-rust - like Ruby-On-Rails -- its a full continous thing


- [x] Remove all warnings in the cargo build

- [x] Actual checks part 1:
  - [x] Are our images and UI exactly same as basecamp campfire - they should be we literally forked from them - check on - https://github.com/basecamp/once-campfire - infact you can find the offline version to compare the assets the font everything directly via going to the folder here - /Users/neetipatni/desktop/Game20250927/once-campfire  - WE SHOULD LOOK AND FEEL Like them
  - [x] https://railway.com/deploy/campfire-rust leads me to nowhere - why was this even allowed to pass


- [-] Actual checks part 2
  - [x] All HTML PAGES that we render and all buttons clicks and interactions are exactly the same as basecamp campfire - they should be we literally forked from them - check on - https://github.com/basecamp/once-campfire - infact you can find the offline version to compare the assets the font everything directly via going to the folder here - /Users/neetipatni/desktop/Game20250927/once-campfire  - WE SHOULD LOOK AND FEEL Like them - List down each page you will ever open + each interaction you will ever have and tell me we same as them - the user should not know if we are not using campfire -- AND constantly document it in a UI parity TIMESTAMP md doc - DO THIS BY BREAKING DOWN THIS TASK INTO COMPREHENSIVE SUBTASKS
    - [x] 1.1 Create comprehensive UI parity analysis document (ui-parity-analysis-20250127.md)
    - [x] 1.2 Extract and analyze original Basecamp Campfire assets from reference directory
    - [x] 1.2.1 Implement LCH color system with semantic abstractions
      - [x] Create colors.css with exact LCH color values from original
      - [x] Implement semantic color abstractions (--color-bg, --color-message-bg, etc.)
      - [x] Add automatic dark mode color transformations
      - [x] Test color accuracy against original side-by-side
      - _Requirements: 13.1, 13.4, 13.5_
    - [x] 1.2.2 Replace Flexbox layout with CSS Grid system
      - [ ] Implement body grid with nav/main/sidebar areas
      - [ ] Add responsive sidebar width system with CSS custom properties
      - [ ] Convert app-container from flex to grid layout
      - [ ] Test layout behavior matches original responsive patterns
      - _Requirements: 13.2, 13.3_
    - [ ] 1.2.3 Implement modular CSS architecture
      - [ ] Split monolithic campfire.css into 25+ modular files
      - [ ] Create base.css with typography and font stack including Aptos
      - [ ] Create layout.css with CSS Grid system
      - [ ] Create messages.css with message grid structure
      - [ ] Create sidebar.css with backdrop blur and positioning
      - [ ] Create composer.css with typing indicators and responsive behavior
      - [ ] Create buttons.css with sophisticated hover system
      - [ ] Create animation.css with original keyframes and timing
      - _Requirements: 17.1, 17.2, 17.3_
    - [ ] 1.3 Message system architecture implementation
      - [ ] 1.3.1 Implement CSS Grid message layout structure
        - [ ] Replace flex-based message layout with CSS Grid
        - [ ] Add grid-template-areas: "sep sep sep" / "avatar body body"
        - [ ] Implement grid-auto-columns with proper spacing variables
        - [ ] Add message column and row gap system
        - _Requirements: 14.1_
      - [ ] 1.3.2 Add day separator system
        - [ ] Implement message__day-separator with grid positioning
        - [ ] Add first-of-day detection logic
        - [ ] Style day separators with original background and border system
        - [ ] Test day separator insertion across multiple days
        - _Requirements: 14.3_
      - [ ] 1.3.3 Implement threaded message support
        - [ ] Add message--threaded class with avatar hiding
        - [ ] Implement subsequent message detection from same user
        - [ ] Add proper margin adjustments for threaded messages
        - [ ] Test threaded message visual behavior
        - _Requirements: 14.2_
      - [ ] 1.3.4 Add message state system
        - [ ] Implement message--failed state with wiggle animation and dashed outline
        - [ ] Add message--mentioned state with LCH-based background highlighting
        - [ ] Implement message--emoji state with large display and transparent background
        - [ ] Add message--formatted visibility system
        - [ ] Test all message states render correctly
        - _Requirements: 14.4, 14.5, 14.6_
      - [ ] 1.3.5 Implement message actions and interactions
        - [ ] Add message options button with original hover behavior
        - [ ] Implement message actions menu with backdrop and arrow styling
        - [ ] Add boost/reaction system placeholder (UI only, no backend)
        - [ ] Implement message editing mode toggle
        - [ ] Test message interaction behavior matches original
        - _Requirements: 14.7_
    - [ ] 1.4 Sidebar and navigation system implementation
      - [ ] 1.4.1 Implement backdrop blur effects
        - [ ] Add -webkit-backdrop-filter and backdrop-filter with 12px blur
        - [ ] Implement proper fallbacks for unsupported browsers
        - [ ] Apply blur to sidebar tools and direct messages sections
        - [ ] Test blur effects across different browsers
        - _Requirements: 15.1_
      - [ ] 1.4.2 Add unread indicators and room states
        - [ ] Implement unread room highlighting with original border system
        - [ ] Add red notification dot to sidebar toggle when unread rooms exist
        - [ ] Implement room state management (current, unread, normal)
        - [ ] Test unread indicator positioning and visibility
        - _Requirements: 15.2, 15.8_
      - [ ] 1.4.3 Implement direct messages horizontal scroll section
        - [ ] Add horizontal scrolling direct messages area
        - [ ] Implement CSS mask for overflow fade effect
        - [ ] Add proper touch scrolling behavior for mobile
        - [ ] Test direct message navigation and overflow handling
        - _Requirements: 15.3_
      - [ ] 1.4.4 Add sidebar responsive behavior
        - [ ] Implement sidebar transform animations for mobile
        - [ ] Add fixed positioning for sidebar tools with backdrop blur
        - [ ] Implement proper z-index layering system
        - [ ] Test sidebar toggle behavior across screen sizes
        - _Requirements: 15.4, 15.5, 15.7_
      - [ ] 1.4.5 Implement room switching and active states
        - [ ] Add active room highlighting with original styling
        - [ ] Implement room switching animations and transitions
        - [ ] Add proper focus management for keyboard navigation
        - [ ] Test room navigation behavior matches original
        - _Requirements: 15.6_
    - [ ] 1.5 Composer and input system implementation
      - [ ] 1.5.1 Implement sophisticated typing indicators
        - [ ] Add typing indicator with CSS custom properties for position and opacity
        - [ ] Implement typing start/stop detection with proper timing
        - [ ] Add spinner animation for active typing state
        - [ ] Test typing indicator positioning and behavior
        - _Requirements: 16.1, 16.5_
      - [ ] 1.5.2 Add responsive composer behavior
        - [ ] Implement context button hiding on mobile when input focused
        - [ ] Add proper viewport handling for mobile keyboards
        - [ ] Implement responsive padding and spacing adjustments
        - [ ] Test composer behavior across different screen sizes
        - _Requirements: 16.2, 16.4, 16.7_
      - [ ] 1.5.3 Add rich text editing foundation (desktop only)
        - [ ] Implement Trix toolbar display logic for desktop
        - [ ] Add rich text mode toggle with original styling
        - [ ] Implement toolbar positioning and styling
        - [ ] Test rich text mode activation and behavior
        - _Requirements: 16.3_
      - [ ] 1.5.4 Implement composer state management
        - [ ] Add composer state transitions with original timing
        - [ ] Implement button show/hide logic with proper easing
        - [ ] Add composer input focus management
        - [ ] Test composer state changes match original behavior
        - _Requirements: 16.6_
    - [ ] 1.6 Asset organization and completeness implementation
      - [ ] 1.6.1 Reorganize image assets into subdirectories
        - [ ] Create browsers/ directory with all browser icons (android.svg, chrome.svg, edge.svg, firefox.svg, opera.svg, safari.svg)
        - [ ] Create external/ directory with integration icons (gear.svg, install.svg, share.svg, sliders.svg, switch.svg, web.svg)
        - [ ] Create logos/ directory with app icons (app-icon-192.png, app-icon.png)
        - [ ] Create screenshots/ directory for mobile documentation images
        - [ ] Test all asset paths and references after reorganization
        - _Requirements: 18.1, 18.2, 18.3, 18.4, 18.5_
      - [ ] 1.6.2 Add sound visualization assets
        - [ ] Create sounds/ image directory for animated GIFs and WebP files
        - [ ] Add visual feedback assets for sound effects (56k.gif, nyan.gif, etc.)
        - [ ] Implement sound visualization display logic
        - [ ] Test sound effect visual feedback system
        - _Requirements: 18.6_
      - [ ] 1.6.3 Update asset references throughout codebase
        - [ ] Update all image src paths to match new directory structure
        - [ ] Update CSS background-image references
        - [ ] Update JavaScript asset loading paths
        - [ ] Test all asset loading works correctly after path updates
        - _Requirements: 18.7_
    - [ ] 1.7 Interactive behavior and accessibility implementation
      - [ ] 1.7.1 Implement sophisticated hover system
        - [ ] Add CSS custom properties for hover color, size, and filter effects
        - [ ] Implement hover state transitions with original timing (150ms ease)
        - [ ] Add brightness filtering and box-shadow effects on hover
        - [ ] Test hover behavior across all interactive elements
        - _Requirements: 19.1_
      - [ ] 1.7.2 Add keyboard navigation and focus management
        - [ ] Implement focus-visible outline system with proper offset calculations
        - [ ] Add keyboard navigation support for all interactive elements
        - [ ] Implement proper focus trap behavior for modals and menus
        - [ ] Test keyboard navigation matches original behavior
        - _Requirements: 19.2_
      - [ ] 1.7.3 Implement animation and transition system
        - [ ] Add original keyframe definitions (wiggle, pulsing-outline, border-fade-out)
        - [ ] Implement transition timing and easing functions
        - [ ] Add animation iteration counts and fill modes
        - [ ] Test all animations match original timing and behavior
        - _Requirements: 19.5_
      - [ ] 1.7.4 Add accessibility and performance features
        - [ ] Implement prefers-reduced-motion media query handling
        - [ ] Add prefers-color-scheme dark mode system with LCH transformations
        - [ ] Ensure 44px minimum touch targets for mobile accessibility
        - [ ] Add proper ARIA labels, roles, and live regions
        - [ ] Test accessibility features with screen readers and keyboard navigation
        - _Requirements: 20.1, 20.2, 20.3, 20.4_
    - [ ] 1.8 Performance optimization and cross-browser compatibility
      - [ ] 1.8.1 Implement performance optimizations
        - [ ] Add CSS layer management for proper reflow optimization
        - [ ] Implement efficient scrollbar styling for cross-browser consistency
        - [ ] Optimize CSS rendering with proper will-change properties
        - [ ] Test performance matches original efficiency benchmarks
        - _Requirements: 20.5_
      - [ ] 1.8.2 Add responsive design system
        - [ ] Implement sophisticated viewport unit usage (dvh, dvw)
        - [ ] Add proper responsive breakpoint system matching original
        - [ ] Implement mobile-first responsive design patterns
        - [ ] Test responsive behavior across all device sizes
        - _Requirements: 20.6_
      - [ ] 1.8.3 Implement cross-browser scrollbar styling
        - [ ] Add WebKit scrollbar styling for Chrome/Safari
        - [ ] Implement Firefox scrollbar-color and scrollbar-width
        - [ ] Add fallbacks for unsupported browsers
        - [ ] Test scrollbar appearance consistency across browsers
        - _Requirements: 20.7_
    - [ ] Update the pre-post analysis in the comprehensive UI parity analysis document (ui-parity-analysis-20250127.md) so we know what changed
    - [ ] Document the changes against the original basecamp implementation refCampfireCodebase
  - [ ] Document differences in User Journeys comprehensively - from each feature difference, to UI experience difference - EVERYTHING via static analysis - where possible estimate - document it in a user journey TIMESTAMP md doc - DO THIS BY BREAKING DOWN THIS TASK INTO COMPREHENSIVE SUBTASKS
    - [ ] 2.1 Create comprehensive user journey analysis document (user-journey-analysis-20250127.md)
    - [ ] 2.2 Core user journey mapping and comparison
      - [ ] 2.2.1 Map first-time user setup journey vs original Campfire
      - [ ] 2.2.2 Map user authentication flow vs original Campfire
      - [ ] 2.2.3 Map main chat experience journey vs original Campfire
      - [ ] 2.2.4 Map room management workflow vs original Campfire
      - [ ] 2.2.5 Map search functionality usage vs original Campfire
    - [ ] 2.3 Advanced feature journey analysis
      - [ ] 2.3.1 Analyze sound system usage patterns vs original
      - [ ] 2.3.2 Analyze push notification setup and usage vs original
      - [ ] 2.3.3 Analyze bot integration workflow vs original
      - [ ] 2.3.4 Analyze file upload/sharing journey (if exists in original)
      - [ ] 2.3.5 Analyze user profile management vs original
    - [ ] 2.4 Administrative journey analysis
      - [ ] 2.4.1 Map admin panel access and functionality vs original
      - [ ] 2.4.2 Map user management workflows vs original
      - [ ] 2.4.3 Map system configuration journeys vs original
      - [ ] 2.4.4 Map room administration workflows vs original
    - [ ] 2.5 Error and edge case journey analysis
      - [ ] 2.5.1 Document error message experiences vs original
      - [ ] 2.5.2 Analyze network failure handling vs original
      - [ ] 2.5.3 Analyze session timeout behavior vs original
      - [ ] 2.5.4 Analyze invalid input handling vs original
    - [ ] 2.6 Performance and behavioral analysis
      - [ ] 2.6.1 Compare response times and loading behaviors
      - [ ] 2.6.2 Analyze real-time update mechanisms vs original
      - [ ] 2.6.3 Compare keyboard shortcuts and hotkeys
      - [ ] 2.6.4 Analyze mobile vs desktop experience differences
    - [ ] 2.7 Feature completeness gap analysis
      - [ ] 2.7.1 Document all missing features from original Campfire
      - [ ] 2.7.2 Document all additional features not in original
      - [ ] 2.7.3 Analyze graceful degradation for missing features
      - [ ] 2.7.4 Document feature behavior differences
    - [ ] 2.8 Critical user experience issues prioritization
      - [ ] 2.8.1 Identify immediate blockers that reveal different product
      - [ ] 2.8.2 Prioritize high-impact user experience differences
      - [ ] 2.8.3 Create action plan for user journey parity fixes
      - [ ] 2.8.4 Define success criteria for user journey validation
 


## Implementation Notes

### Development Approach
- Fix compilation errors manually, don't build automated systems
- Test every change on clean environments
- Prioritize working software over perfect code
- Use simple solutions over complex architectures

### Testing Strategy
- Test each installation method on clean machines
- Verify every command in documentation works
- Focus on user experience, not just technical correctness
- Get feedback from real users during development

### Shreyas Doshi Principles Applied
- Remove friction, don't add complexity
- Get users to working software immediately
- Be honest about what works vs what doesn't
- Focus on deployment success, not vanity metrics
- Two clear paths, no choice paralysis