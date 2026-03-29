# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | Yes                |

## Reporting a Vulnerability

If you discover a security vulnerability in Ferrum Email, please report it responsibly.

**Do NOT open a public GitHub issue for security vulnerabilities.**

Instead, email us at: **security@automatanexus.com**

Include:
- Description of the vulnerability
- Steps to reproduce
- Impact assessment
- Suggested fix (if any)

We will acknowledge your report within 48 hours and provide a timeline for a fix within 5 business days.

## Security Measures

- All SMTP connections use STARTTLS
- Passwords hashed with Argon2id
- JWT tokens with configurable expiry
- NexusShield integration for inbound/outbound email validation
- Input validation on all email fields (RFC 5322 compliance)
- Size limits on subjects, bodies, attachments, and recipient counts
- Domain sanitization for DNS lookups
