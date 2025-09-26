# Security Policy

## Supported Versions

We actively support the following versions of Campfire with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | ✅ Yes             |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report security vulnerabilities to us privately:

### 📧 Email
Send details to: **security@that-in-rust.dev**

### 📋 What to Include
Please include as much information as possible:

- **Type of issue** (e.g. buffer overflow, SQL injection, cross-site scripting, etc.)
- **Full paths** of source file(s) related to the manifestation of the issue
- **Location** of the affected source code (tag/branch/commit or direct URL)
- **Special configuration** required to reproduce the issue
- **Step-by-step instructions** to reproduce the issue
- **Proof-of-concept or exploit code** (if possible)
- **Impact** of the issue, including how an attacker might exploit it

### 🔒 Our Commitment

- **Response Time**: We'll acknowledge receipt within 48 hours
- **Investigation**: We'll investigate and respond within 5 business days
- **Updates**: We'll keep you informed of our progress
- **Credit**: We'll credit you in our security advisory (if desired)

### 🛡️ Security Measures

Campfire includes several built-in security features:

- **bcrypt password hashing** with secure salts
- **Secure session tokens** with proper expiration
- **Built-in rate limiting** to prevent abuse
- **Input validation** and sanitization
- **HTTPS enforcement** in production
- **CSRF protection** for state-changing operations
- **SQL injection prevention** through parameterized queries

### 🚨 Vulnerability Disclosure Policy

1. **Private Disclosure**: Report to us privately first
2. **Investigation Period**: We'll investigate within 5 business days
3. **Fix Development**: We'll develop and test a fix
4. **Coordinated Release**: We'll coordinate the public disclosure
5. **Public Advisory**: We'll publish a security advisory with details

### 🏆 Recognition

We believe in recognizing security researchers who help keep Campfire secure:

- **Hall of Fame**: Security researchers who report valid vulnerabilities
- **Public Credit**: In security advisories and release notes (if desired)
- **Swag**: Campfire stickers and merchandise for significant findings

### 📚 Security Resources

- **Security Best Practices**: See our [deployment documentation](docs/)
- **Secure Configuration**: Follow our production deployment guides
- **Updates**: Subscribe to our releases for security updates

### 🤝 Responsible Disclosure

We ask that you:

- **Give us reasonable time** to investigate and fix issues before public disclosure
- **Avoid privacy violations**, destruction of data, or service disruption
- **Only interact with accounts you own** or have explicit permission to access
- **Don't access or modify data** belonging to other users

Thank you for helping keep Campfire and our users safe! 🔥