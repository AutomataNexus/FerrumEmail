# Changelog

All notable changes to the Ferrum Email SDK are documented here.

## [0.1.3] - 2026-03-30

### Added
- `DirectMxProvider` — direct MX delivery without third-party relay (NexusRelay)
- `VaultCredentialStore` re-exported from prelude
- RFC 5322 email address validation with domain/TLD checks
- Input size limits: subject (998 chars), body (10MB), attachments (24MB), recipients (500)
- Domain sanitization in DirectMxProvider before DNS lookups
- `SECURITY.md` with vulnerability disclosure policy
- Keywords, categories, homepage metadata on all crates

### Changed
- Vault passphrase read from `FERRUM_VAULT_PASSPHRASE` env var (no longer hardcoded)
- TUI aligned to workspace version/edition
- Removed stub crates (CLI, preview, macros) from workspace and release workflow

### Fixed
- Zero clippy warnings across entire workspace
- README quick-start now includes `tokio` dependency
- All public types properly exported and accessible via prelude

## [0.1.2] - 2026-03-30

### Added
- Security hardening and metadata updates
- SLSA provenance attestation on release builds

## [0.1.1] - 2026-03-26

### Added
- Native SMTP provider with STARTTLS (RFC 5321, 3207, 4954)
- AUTH PLAIN and AUTH LOGIN support
- MIME multipart message formatting (RFC 2045)
- NexusShield integration for inbound/outbound email validation
- NexusVault integration for encrypted credential storage
- TUI dashboard with template preview, compose, and vault integration

## [0.1.0] - 2026-03-25

### Added
- Initial release
- Component-based email framework for Rust
- Core types: Component trait, Node tree, Style system
- 14 standard components: Html, Body, Button, Text, Heading, Link, Image, etc.
- HTML renderer with CSS inlining
- Plain text extraction
- ConsoleProvider for development
- Two example templates (welcome, password-reset)
