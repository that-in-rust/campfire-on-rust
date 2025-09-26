# Contributing to campfire-on-rust 🔥

Thank you for your interest in contributing to campfire-on-rust! This guide will help you get started with contributing to our Rust-powered team chat application.

## 🚀 Quick Start

### Prerequisites
- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **Git**: For version control
- **A code editor**: VS Code with rust-analyzer recommended

### Development Setup
```bash
# Clone the repository
git clone https://github.com/that-in-rust/campfire-on-rust.git
cd campfire-on-rust

# Build and run
cargo run

# Run tests
cargo test

# Check code quality
cargo clippy
cargo fmt
```

## 🎯 Ways to Contribute

### 🐛 Bug Reports
**Found a bug?** Help us fix it!

1. **Check existing issues**: Search [GitHub Issues](https://github.com/that-in-rust/campfire-on-rust/issues)
2. **Create detailed report**: Use our bug report template
3. **Include**: OS, browser, steps to reproduce, expected vs actual behavior

### 💡 Feature Requests
**Have an idea?** We'd love to hear it!

1. **Check discussions**: Browse [GitHub Discussions](https://github.com/that-in-rust/campfire-on-rust/discussions)
2. **Describe the problem**: What user need does this address?
3. **Propose solution**: How should it work?
4. **Consider scope**: Does it fit campfire-on-rust's simplicity philosophy?

### 🔧 Code Contributions
**Ready to code?** Here's how:

1. **Find an issue**: Look for "good first issue" or "help wanted" labels
2. **Discuss first**: Comment on the issue to coordinate
3. **Fork and branch**: Create a feature branch from `main`
4. **Write tests**: Follow TDD principles (test first!)
5. **Submit PR**: Use our pull request template

## 📋 Development Guidelines

### Code Style
- **Format**: Use `cargo fmt` (rustfmt)
- **Lint**: Pass `cargo clippy` with no warnings
- **Test**: Write tests for new functionality
- **Document**: Add doc comments for public APIs

### Architecture Principles
campfire-on-rust follows these core principles:

1. **Simplicity over complexity**: Core features only, no bloat
2. **Performance**: Rust's zero-cost abstractions
3. **Reliability**: Comprehensive error handling
4. **Security**: Safe defaults, input validation
5. **Testability**: Dependency injection, trait abstractions

### Testing Strategy
- **Unit tests**: Test individual functions and modules
- **Integration tests**: Test component interactions
- **End-to-end tests**: Test complete user workflows
- **Performance tests**: Validate timing and memory claims

### Commit Messages
Use conventional commits format:
```
type(scope): description

feat(chat): add @mention notifications
fix(auth): resolve session timeout issue
docs(readme): update installation instructions
test(api): add webhook integration tests
```

## 🏗️ Project Structure

```
campfire-on-rust/
├── src/
│   ├── handlers/          # HTTP request handlers
│   ├── services/          # Business logic
│   ├── models/            # Data structures
│   ├── middleware/        # HTTP middleware
│   └── main.rs           # Application entry point
├── templates/            # HTML templates
├── assets/              # Static assets (CSS, JS, sounds)
├── tests/               # Integration tests
├── scripts/             # Deployment and utility scripts
└── docs/                # Documentation
```

## 🎯 Contribution Areas

### High Priority
- **Performance optimization**: Memory usage, startup time
- **Security enhancements**: Rate limiting, input validation
- **Mobile experience**: Responsive design improvements
- **Accessibility**: WCAG compliance, keyboard navigation

### Medium Priority
- **File attachments**: Image and document sharing
- **Search improvements**: Advanced filters, highlighting
- **Bot integrations**: Webhook enhancements, API expansion
- **Deployment options**: Docker improvements, cloud templates

### Future Features
- **Voice/video calls**: WebRTC integration
- **Native mobile apps**: React Native or Flutter
- **Enterprise features**: SSO, audit logs, compliance
- **Advanced analytics**: Usage metrics, performance monitoring

## 🔍 Code Review Process

### For Contributors
1. **Self-review**: Check your own code first
2. **Test thoroughly**: Run full test suite
3. **Update docs**: Keep documentation current
4. **Small PRs**: Easier to review and merge

### Review Criteria
- **Functionality**: Does it work as intended?
- **Performance**: No significant regressions
- **Security**: No new vulnerabilities
- **Style**: Follows project conventions
- **Tests**: Adequate test coverage
- **Documentation**: Clear and complete

## 🚨 Getting Help

### Development Questions
- **GitHub Discussions**: [Ask the community](https://github.com/that-in-rust/campfire-on-rust/discussions)
- **Discord**: Join our development chat (link in README)
- **Email**: [dev@that-in-rust.dev](mailto:dev@that-in-rust.dev)

### Stuck on Something?
1. **Check documentation**: README, code comments, tests
2. **Search issues**: Someone might have faced this before
3. **Ask questions**: No question is too basic!
4. **Pair programming**: We're happy to help via video call

## 📜 Code of Conduct

### Our Standards
- **Be respectful**: Treat everyone with kindness and respect
- **Be inclusive**: Welcome people of all backgrounds and experience levels
- **Be constructive**: Focus on helping and improving
- **Be patient**: Remember everyone is learning

### Unacceptable Behavior
- Harassment, discrimination, or offensive language
- Personal attacks or trolling
- Spam or off-topic discussions
- Sharing private information without permission

### Enforcement
Issues will be addressed by project maintainers. Serious violations may result in temporary or permanent bans from the project.

## 🎉 Recognition

### Contributors
All contributors are recognized in:
- **README**: Contributors section
- **Releases**: Changelog acknowledgments
- **Discord**: Special contributor role
- **Swag**: Stickers and t-shirts for significant contributions

### Maintainers
Regular contributors may be invited to become maintainers with:
- **Commit access**: Direct push to repository
- **Review rights**: Approve and merge pull requests
- **Issue triage**: Label and prioritize issues
- **Release management**: Cut releases and manage deployments

## 📚 Resources

### Learning Rust
- **The Rust Book**: [doc.rust-lang.org/book](https://doc.rust-lang.org/book/)
- **Rust by Example**: [doc.rust-lang.org/rust-by-example](https://doc.rust-lang.org/rust-by-example/)
- **Rustlings**: Interactive exercises

### Web Development
- **Axum**: Web framework documentation
- **SQLx**: Database toolkit
- **Tokio**: Async runtime

### Project-Specific
- **Architecture docs**: `/docs/architecture.md`
- **API documentation**: `/docs/api.md`
- **Deployment guide**: `/docs/deployment.md`

## 🙏 Thank You

Every contribution, no matter how small, makes campfire-on-rust better for everyone. Whether you're fixing a typo, reporting a bug, or implementing a major feature, your help is appreciated!

**Happy coding!** 🦀🔥