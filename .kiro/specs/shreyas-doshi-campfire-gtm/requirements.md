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