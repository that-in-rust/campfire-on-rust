# Requirements Document: Shreyas Doshi Campfire GTM

## Introduction

This feature creates a Shreyas Doshi-inspired Go-To-Market onboarding experience for Campfire that focuses on clear user segmentation, low-friction adoption paths, and value-driven messaging. The goal is to transform GitHub visitors into active users through strategic onboarding flows that match different user intents and contexts.

Shreyas Doshi's approach emphasizes:
- Clear user segments with distinct needs
- Friction-reducing onboarding paths
- Value demonstration before feature explanation
- Progressive disclosure of complexity
- Measurable conversion funnels

## Requirements

### Requirement 1: Single-Focus User Experience

**User Story:** As a GitHub visitor exploring Campfire, I want to immediately understand how to get team chat working for my team, so that I can deploy it right now without confusion.

#### Acceptance Criteria

1. WHEN a user visits the GitHub repository THEN they SHALL see one primary action: "Deploy Campfire Now"
2. WHEN a user sees the main page THEN the primary deployment button SHALL be the most prominent element
3. WHEN a user wants to see a demo first THEN they SHALL find a small secondary link that leads back to deployment
4. WHEN a user wants to run locally THEN they SHALL find a small secondary link with working instructions
5. WHEN a user completes any secondary action THEN they SHALL be guided back to the primary deployment path

### Requirement 2: Simple Demo Experience

**User Story:** As a potential user who wants to see Campfire working before deploying, I want a quick demo that shows me the product and then gets me to deployment, so that I can make a fast decision.

#### Acceptance Criteria

1. WHEN a user clicks "See it working first" THEN they SHALL get a working demo within 3 seconds
2. WHEN in demo mode THEN it SHALL be obvious this is a demo with realistic team chat data
3. WHEN a user tries the demo THEN they SHALL see a prominent "Deploy for Real" button
4. WHEN a user finishes exploring the demo THEN they SHALL be directed to the deployment button
5. WHEN demo mode is enabled THEN it SHALL work without any setup or configuration

### Requirement 3: Single Primary Deployment Path

**User Story:** As a team decision-maker, I want one clear way to deploy Campfire that works reliably, so that I can get my team using it immediately without evaluating multiple options.

#### Acceptance Criteria

1. WHEN a user clicks "Deploy Campfire Now" THEN they SHALL be taken to a working Railway deployment
2. WHEN the Railway deployment starts THEN it SHALL complete successfully within 3 minutes
3. WHEN deployment completes THEN the user SHALL have a working Campfire instance with a URL
4. WHEN a user prefers local installation THEN they SHALL find working instructions as a secondary option
5. WHEN any deployment method is provided THEN it SHALL be tested and verified to work before publishing
6. WHEN deployment fails THEN the user SHALL get clear error messages and working alternatives

### Requirement 4: Clear Value Communication

**User Story:** As a user exploring Campfire, I want to immediately understand what it does and why I should use it, so that I can decide quickly whether to deploy it.

#### Acceptance Criteria

1. WHEN a user visits the repository THEN they SHALL immediately see "Team chat that works" or similar clear value proposition
2. WHEN describing features THEN the documentation SHALL focus on what teams get, not technical implementation
3. WHEN comparing to alternatives THEN it SHALL be honest about what Campfire does well vs what it doesn't do
4. WHEN a user reads the README THEN they SHALL understand the value within 30 seconds
5. WHEN technical details are needed THEN they SHALL be available but not prominent

### Requirement 5: Simple Success Tracking

**User Story:** As a product team, I want to know if people are successfully deploying and using Campfire, so that we can improve what's not working.

#### Acceptance Criteria

1. WHEN a user clicks "Deploy Now" THEN we SHALL track that click
2. WHEN a deployment completes successfully THEN we SHALL track that success
3. WHEN a deployment fails THEN we SHALL track the failure and reason
4. WHEN users are getting stuck THEN we SHALL identify the most common failure points
5. WHEN tracking data is collected THEN it SHALL focus on deployment success, not complex user journeys

### Requirement 6: Honest Credibility

**User Story:** As a cautious adopter, I want to trust that Campfire actually works and isn't just marketing hype, so that I can feel confident deploying it.

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

**User Story:** As a mobile user discovering Campfire, I want to understand what it is and be able to deploy it, so that I can get my team using it even when I'm on my phone.

#### Acceptance Criteria

1. WHEN accessing on mobile THEN the README SHALL be readable and the "Deploy Now" button SHALL be easily tappable
2. WHEN using mobile THEN the demo SHALL work and be touch-friendly
3. WHEN deploying from mobile THEN the Railway deployment process SHALL work on mobile browsers
4. WHEN mobile deployment is difficult THEN users SHALL be able to email themselves the deployment link
5. WHEN the deployed Campfire is accessed on mobile THEN it SHALL be responsive and usable

### Requirement 9: Code Compilation and Basic Functionality

**User Story:** As a developer trying Campfire for the first time, I want the code to compile and run successfully, so that I can evaluate the product without encountering basic technical failures.

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

**User Story:** As a developer trying Campfire for the first time, I want every installation command to work exactly as documented, so that I don't lose confidence in the product before I even start.

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

1. WHEN viewing the repository root THEN users SHALL see only essential files that directly relate to understanding, building, or deploying Campfire
2. WHEN development artifacts exist THEN they SHALL be moved to appropriate hidden directories (.kiro/, .github/, etc.) or archived locations
3. WHEN documentation exists THEN it SHALL be consolidated into a clear hierarchy with no duplicate or outdated files
4. IF task summaries or development logs are needed THEN they SHALL be moved to development-specific locations not visible to end users
5. WHEN the repository is viewed THEN it SHALL present the same clean, focused impression that DHH and Jason Fried would expect from a Basecamp-quality product
6. WHEN files like .Slate/, monitoring/, benches/, multiple TASK_*.md files are present THEN they SHALL be relocated to development-only areas
7. WHEN multiple similar files exist (DEPLOYMENT.md, deployment-guide.md, etc.) THEN they SHALL be consolidated into single, authoritative versions