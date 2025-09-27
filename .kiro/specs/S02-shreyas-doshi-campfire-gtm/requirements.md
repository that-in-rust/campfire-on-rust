# Requirements Document: Shreyas Doshi campfire-on-rust GTM

## Introduction

This feature creates a Shreyas Doshi-inspired Go-To-Market onboarding experience for campfire-on-rust that focuses on clear user segmentation, low-friction adoption paths, and value-driven messaging. The goal is to transform GitHub visitors into active users through strategic onboarding flows that match different user intents and contexts.

Shreyas Doshi's approach emphasizes:
- Clear user segments with distinct needs
- Friction-reducing onboarding paths
- Value demonstration before feature explanation
- Progressive disclosure of complexity
- Measurable conversion funnels

## Requirements

### Requirement 1: Two-Path User Experience

**User Story:** As a GitHub visitor exploring campfire-on-rust, I want to either try it locally or deploy it for my team, so that I can get to working software immediately without confusion.

#### Acceptance Criteria

1. WHEN a user visits the GitHub repository THEN they SHALL see two clear options: "Try it locally" and "Deploy for your team"
2. WHEN a user chooses "Try it locally" THEN they SHALL get a working curl command that downloads and starts campfire-on-rust immediately
3. WHEN a user chooses "Deploy for your team" THEN they SHALL get a working Railway deployment button
4. WHEN a user successfully runs locally THEN they SHALL see a clear path to deploy for their team
5. WHEN either path is chosen THEN the user SHALL get to working campfire-on-rust software within 3 minutes

### Requirement 2: Local Sampling Experience

**User Story:** As a developer who wants to try campfire-on-rust before deploying it for my team, I want to run it locally and see it working, so that I can evaluate it properly.

#### Acceptance Criteria

1. WHEN a user runs the install script THEN campfire-on-rust SHALL download, install, and start automatically on localhost:3000
2. WHEN campfire-on-rust runs locally THEN it SHALL have demo data that shows realistic team chat functionality
3. WHEN a user explores the local version THEN they SHALL see all core features working (chat, rooms, search)
4. WHEN a user is satisfied with local testing THEN they SHALL see a clear "Deploy for Your Team" option
5. WHEN local installation is provided THEN it SHALL work without complex setup or configuration

### Requirement 3: Team Deployment Path

**User Story:** As a team decision-maker, I want to deploy campfire-on-rust for my team quickly and reliably, so that we can start using it for team chat immediately.

#### Acceptance Criteria

1. WHEN a user clicks "Deploy for Your Team" THEN they SHALL be taken to a working Railway deployment
2. WHEN the Railway deployment starts THEN it SHALL complete successfully within 3 minutes
3. WHEN deployment completes THEN the user SHALL have a working campfire-on-rust instance with a URL they can share
4. WHEN team members visit the deployed URL THEN they SHALL be able to create accounts and start chatting
5. WHEN deployment is provided THEN it SHALL be tested and verified to work before publishing
6. WHEN deployment fails THEN the user SHALL get clear error messages and support contact information

### Requirement 4: Clear Value Communication

**User Story:** As a user exploring campfire-on-rust, I want to immediately understand what it does and why I should use it, so that I can decide quickly whether to deploy it.

#### Acceptance Criteria

1. WHEN a user visits the repository THEN they SHALL immediately see "Team chat that works" or similar clear value proposition
2. WHEN describing features THEN the documentation SHALL focus on what teams get, not technical implementation
3. WHEN comparing to alternatives THEN it SHALL be honest about what campfire-on-rust does well vs what it doesn't do
4. WHEN a user reads the README THEN they SHALL understand the value within 30 seconds
5. WHEN technical details are needed THEN they SHALL be available but not prominent

### Requirement 5: Simple Success Tracking

**User Story:** As a product team, I want to know if people are successfully deploying and using campfire-on-rust, so that we can improve what's not working.

#### Acceptance Criteria

1. WHEN a user clicks "Deploy Now" THEN we SHALL track that click
2. WHEN a deployment completes successfully THEN we SHALL track that success
3. WHEN a deployment fails THEN we SHALL track the failure and reason
4. WHEN users are getting stuck THEN we SHALL identify the most common failure points
5. WHEN tracking data is collected THEN it SHALL focus on deployment success, not complex user journeys

### Requirement 6: Honest Credibility

**User Story:** As a cautious adopter, I want to trust that campfire-on-rust actually works and isn't just marketing hype, so that I can feel confident deploying it.

#### Acceptance Criteria

1. WHEN a user views the main page THEN they SHALL see honest information about what works and what doesn't
2. WHEN showing metrics THEN they SHALL be real, measured numbers, not marketing claims
3. WHEN describing the product THEN it SHALL be clear this is an MVP with core features, not enterprise software
4. WHEN users have questions THEN they SHALL find real contact information and support channels
5. WHEN social proof is limited THEN the focus SHALL be on product transparency and working demos

### Requirement 7: Simple Help When Needed

**User Story:** As a user encountering issues during deployment, I want clear help that gets me back on track quickly, so that I can complete my deployment successfully.

#### Acceptance Criteria

1. WHEN deployment fails THEN the user SHALL get a clear error message explaining what went wrong
2. WHEN users need help THEN they SHALL find simple troubleshooting steps for common issues
3. WHEN troubleshooting doesn't work THEN users SHALL find real contact information for support
4. WHEN common problems occur THEN they SHALL be documented with solutions in the README
5. WHEN help is provided THEN it SHALL focus on getting deployment working, not complex diagnostics

### Requirement 8: Mobile-Friendly Experience

**User Story:** As a mobile user discovering campfire-on-rust, I want to understand what it is and be able to deploy it, so that I can get my team using it even when I'm on my phone.

#### Acceptance Criteria

1. WHEN accessing on mobile THEN the README SHALL be readable and the "Deploy Now" button SHALL be easily tappable
2. WHEN using mobile THEN the demo SHALL work and be touch-friendly
3. WHEN deploying from mobile THEN the Railway deployment process SHALL work on mobile browsers
4. WHEN mobile deployment is difficult THEN users SHALL be able to email themselves the deployment link
5. WHEN the deployed campfire-on-rust is accessed on mobile THEN it SHALL be responsive and usable

### Requirement 9: Code Compilation and Basic Functionality

**User Story:** As a developer trying campfire-on-rust for the first time, I want the code to compile and run successfully, so that I can evaluate the product without encountering basic technical failures.

#### Acceptance Criteria

1. WHEN a user runs `cargo check` THEN the code SHALL compile without any errors
2. WHEN a user runs `cargo run` THEN the application SHALL start successfully and be accessible at http://localhost:3000
3. WHEN the current codebase has 41 compilation errors THEN they SHALL be fixed before any GTM activities
4. WHEN enum variants are referenced (like `TypingIndicator`, `TooManyConnections`) THEN they SHALL exist in the actual enum definitions
5. WHEN type mismatches exist (u64 vs usize, Arc<T> vs T) THEN they SHALL be resolved for consistent typing
6. WHEN structs need trait implementations (Clone, Debug, Serialize, Deserialize) THEN they SHALL be properly implemented
7. WHEN lifetime issues prevent compilation THEN they SHALL be resolved with proper lifetime management
8. WHEN thread safety issues exist THEN they SHALL be resolved to ensure Send + Sync compliance where needed

### Requirement 10: Installation Command Reliability

**User Story:** As a developer trying campfire-on-rust for the first time, I want every installation command to work exactly as documented, so that I don't lose confidence in the product before I even start.

#### Acceptance Criteria

1. WHEN any installation command is documented THEN it SHALL be tested on a clean machine before publication
2. WHEN the current README install command uses placeholder URLs THEN it SHALL be updated to use actual repository URLs
3. WHEN install scripts reference pre-built binaries THEN those binaries SHALL actually exist and be downloadable
4. WHEN users copy-paste install commands THEN they SHALL work without modification on macOS, Linux, and Windows where applicable
5. IF no one-command install is ready THEN the README SHALL provide the actual working commands that have been verified to work
6. WHEN installation commands are updated THEN they SHALL be tested by someone other than the author on a fresh environment
7. WHEN any installation method fails THEN there SHALL be immediate fallback instructions that definitely work

### Requirement 11: Repository Cleanliness and Focus

**User Story:** As a potential user or contributor browsing the GitHub repository, I want to see a clean, focused codebase that demonstrates professionalism and clarity, so that I can quickly understand the project's value and quality.

#### Acceptance Criteria

1. WHEN viewing the repository root THEN users SHALL see only essential files that directly relate to understanding, building, or deploying campfire-on-rust
2. WHEN development artifacts exist THEN they SHALL be moved to appropriate hidden directories (.kiro/, .github/, etc.) or archived locations
3. WHEN documentation exists THEN it SHALL be consolidated into a clear hierarchy with no duplicate or outdated files
4. IF task summaries or development logs are needed THEN they SHALL be moved to development-specific locations not visible to end users
5. WHEN the repository is viewed THEN it SHALL present the same clean, focused impression that DHH and Jason Fried would expect from a Basecamp-quality product
6. WHEN files like .Slate/, monitoring/, benches/, multiple TASK_*.md files are present THEN they SHALL be relocated to development-only areas
7. WHEN multiple similar files exist (DEPLOYMENT.md, deployment-guide.md, etc.) THEN they SHALL be consolidated into single, authoritative versions

### Requirement 12: Professional CI/CD Testing Architecture

**User Story:** As a developer maintaining campfire-on-rust's release process, I want a professional testing framework that validates all CI/CD functionality automatically, so that I can confidently deploy releases without manual verification or custom bash scripts.

#### Acceptance Criteria

1. WHEN testing GitHub Actions workflows THEN the system SHALL use `act` or equivalent professional tools to validate workflows locally before deployment
2. WHEN testing cross-platform builds THEN the system SHALL use `cargo-dist` or equivalent Rust-native tools for reliable multi-platform binary generation
3. WHEN testing installation processes THEN the system SHALL use `testcontainers` or equivalent infrastructure testing frameworks to simulate clean environments
4. WHEN validating release artifacts THEN the system SHALL use `goss` or equivalent server testing tools to verify binary functionality and deployment success
5. WHEN testing CI/CD pipelines THEN custom bash scripts SHALL be replaced with industry-standard testing frameworks that provide structured reporting and debugging
6. WHEN integration testing is required THEN the system SHALL use proper Rust testing frameworks (`tokio-test`, `testcontainers-rs`) rather than shell script simulations
7. WHEN performance contracts need validation THEN the system SHALL use `criterion` benchmarks with automated regression detection rather than manual timing checks

### Requirement 13: Visual Design System Parity

**User Story:** As a user comparing campfire-on-rust to the original Basecamp Campfire, I want the visual design to be indistinguishable from the original, so that I get the authentic Basecamp experience I expect.

#### Acceptance Criteria

1. WHEN viewing the color scheme THEN campfire-on-rust SHALL use the original LCH color space system with semantic abstractions, not basic RGB colors
2. WHEN the interface renders THEN it SHALL use CSS Grid layout matching the original's sophisticated grid system, not simplified Flexbox
3. WHEN typography displays THEN it SHALL include the complete font stack including Aptos and emoji fonts, matching the original exactly
4. WHEN dark mode is enabled THEN colors SHALL transform using the original's LCH-based automatic color transformations
5. WHEN message backgrounds render THEN they SHALL use the original's `--color-message-bg` system with proper semantic color mapping
6. WHEN comparing side-by-side with original Campfire THEN a user SHALL not be able to distinguish visual differences in the core interface
7. WHEN CSS custom properties are used THEN they SHALL match the original's naming conventions and value calculations exactly

### Requirement 14: Message System Architecture Parity

**User Story:** As a user interacting with messages in campfire-on-rust, I want the message system to behave exactly like the original Basecamp Campfire, so that I get the familiar and polished experience.

#### Acceptance Criteria

1. WHEN messages display THEN they SHALL use CSS Grid layout with proper grid-template-areas matching the original's "sep sep sep" / "avatar body body" structure
2. WHEN multiple messages from the same user appear THEN subsequent messages SHALL display as threaded messages without repeating avatars
3. WHEN a new day begins THEN messages SHALL display day separators with the original's styling and positioning
4. WHEN messages fail to send THEN they SHALL display with the original's failed message styling including wiggle animation and dashed outline
5. WHEN messages are mentioned THEN they SHALL display with the original's mention highlighting using LCH-based background colors
6. WHEN emoji-only messages are sent THEN they SHALL display with the original's large emoji styling and transparent background
7. WHEN message actions are available THEN they SHALL appear with the original's hover behavior and positioning system
8. WHEN messages have different states THEN they SHALL support pending, failed, threaded, and mentioned states with original styling

### Requirement 15: Sidebar and Navigation Parity

**User Story:** As a user navigating campfire-on-rust, I want the sidebar and navigation to work exactly like the original Basecamp Campfire, so that I can navigate efficiently with the familiar interface.

#### Acceptance Criteria

1. WHEN the sidebar displays THEN it SHALL use backdrop-filter blur effects matching the original's 12px blur with proper fallbacks
2. WHEN rooms are listed THEN they SHALL display unread indicators with the original's styling and positioning
3. WHEN direct messages appear THEN they SHALL display in the horizontal scrolling section with proper masking and overflow handling
4. WHEN the sidebar toggles THEN it SHALL use the original's transform animations and positioning system
5. WHEN sidebar tools display THEN they SHALL use fixed positioning with backdrop blur matching the original exactly
6. WHEN room switching occurs THEN active room highlighting SHALL match the original's border and background system
7. WHEN mobile view is active THEN sidebar SHALL transform and position exactly like the original's mobile behavior
8. WHEN unread rooms exist THEN the sidebar toggle SHALL display the original's red notification dot with proper positioning

### Requirement 16: Composer and Input System Parity

**User Story:** As a user composing messages in campfire-on-rust, I want the message composer to work exactly like the original Basecamp Campfire, so that I can write messages with the same familiar interface and capabilities.

#### Acceptance Criteria

1. WHEN typing indicators display THEN they SHALL use the original's sophisticated positioning system with CSS custom properties for position and opacity
2. WHEN the composer input focuses THEN context buttons SHALL hide on mobile exactly like the original's responsive behavior
3. WHEN rich text mode is available THEN it SHALL display Trix toolbar with the original's styling and behavior (desktop only)
4. WHEN the composer renders THEN it SHALL use the original's padding and spacing system with proper responsive adjustments
5. WHEN typing detection occurs THEN it SHALL implement the original's typing start/stop logic with proper timing
6. WHEN composer state changes THEN buttons SHALL show/hide with the original's transition timing and easing
7. WHEN mobile keyboards appear THEN the composer SHALL adjust with the original's viewport handling and positioning

### Requirement 17: CSS Architecture Modernization

**User Story:** As a developer maintaining campfire-on-rust's styles, I want a modular CSS architecture matching the original Basecamp Campfire, so that styles are maintainable and match the original's sophisticated organization.

#### Acceptance Criteria

1. WHEN CSS files are organized THEN they SHALL be split into 25+ modular files matching the original's architecture (colors.css, layout.css, messages.css, etc.)
2. WHEN CSS custom properties are defined THEN they SHALL use the original's naming conventions and semantic abstractions
3. WHEN responsive design is implemented THEN it SHALL use the original's sophisticated media query system and viewport units
4. WHEN animations are defined THEN they SHALL match the original's timing, easing, and keyframe definitions exactly
5. WHEN CSS Grid is used THEN it SHALL implement the original's grid-template-areas and responsive grid systems
6. WHEN hover states are defined THEN they SHALL use the original's sophisticated hover system with CSS custom properties
7. WHEN the CSS compiles THEN it SHALL produce the same visual output as the original when rendered side-by-side

### Requirement 18: Asset Organization and Completeness

**User Story:** As a user and developer of campfire-on-rust, I want all assets to be organized and complete like the original Basecamp Campfire, so that the application has the same professional polish and functionality.

#### Acceptance Criteria

1. WHEN images are organized THEN they SHALL be structured in subdirectories matching the original (browsers/, external/, logos/, screenshots/, sounds/)
2. WHEN browser icons are needed THEN the browsers/ directory SHALL contain all original SVG icons (android.svg, chrome.svg, edge.svg, firefox.svg, opera.svg, safari.svg)
3. WHEN external integration icons are needed THEN the external/ directory SHALL contain all original icons (gear.svg, install.svg, share.svg, sliders.svg, switch.svg, web.svg)
4. WHEN app logos are needed THEN the logos/ directory SHALL contain the original app icons (app-icon-192.png, app-icon.png)
5. WHEN documentation images are needed THEN the screenshots/ directory SHALL contain mobile app screenshots for documentation
6. WHEN sound effects play THEN they SHALL optionally display the original's animated GIFs and WebP files for visual feedback
7. WHEN assets are referenced THEN they SHALL use the same path structure and naming conventions as the original

### Requirement 19: Interactive Behavior Parity

**User Story:** As a user interacting with campfire-on-rust, I want all interactive behaviors to match the original Basecamp Campfire exactly, so that muscle memory and expectations from the original work seamlessly.

#### Acceptance Criteria

1. WHEN hovering over interactive elements THEN they SHALL use the original's sophisticated hover system with CSS custom properties for color, size, and filter effects
2. WHEN keyboard navigation is used THEN focus states SHALL match the original's outline system with proper offset and color calculations
3. WHEN elements are pressed THEN they SHALL use the original's active state styling and transition timing
4. WHEN disabled states are shown THEN they SHALL match the original's brightness filtering and cursor behavior
5. WHEN animations play THEN they SHALL use the original's keyframe definitions, timing functions, and iteration counts
6. WHEN scroll behavior occurs THEN it SHALL match the original's overscroll-behavior and scroll masking systems
7. WHEN touch interactions happen THEN they SHALL use the original's touch-action and responsive behavior patterns

### Requirement 20: Performance and Accessibility Parity

**User Story:** As a user with accessibility needs or performance constraints, I want campfire-on-rust to meet the same standards as the original Basecamp Campfire, so that I can use it effectively regardless of my capabilities or device.

#### Acceptance Criteria

1. WHEN reduced motion is preferred THEN animations SHALL be disabled matching the original's prefers-reduced-motion media query handling
2. WHEN color scheme preferences exist THEN dark mode SHALL activate using the original's prefers-color-scheme system with LCH color transformations
3. WHEN touch targets are needed THEN they SHALL meet the original's 44px minimum size requirements for accessibility
4. WHEN screen readers are used THEN semantic markup SHALL match the original's ARIA labels, roles, and live regions
5. WHEN performance is measured THEN CSS rendering SHALL match the original's efficiency with proper layer management and reflow optimization
6. WHEN viewport changes occur THEN responsive behavior SHALL match the original's sophisticated viewport unit usage and responsive design
7. WHEN scrollbars are styled THEN they SHALL use the original's cross-browser scrollbar styling for consistent appearance