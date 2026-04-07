# Changelog

All notable changes to the Ferrum Email SDK are documented here.

## [0.1.5] - 2026-04-06

### Added
- **NexusRelay v0.2** — direct MX delivery with DKIM signing, port 25/587 fallback
- `DirectMxProvider` now tries port 25 first, falls back to 587 for cloud SMTP compatibility
- Domain validation and sanitization before DNS MX lookups (prevents command injection)
- 15s connect / 30s command / 60s data timeouts on all outbound SMTP operations
- `SmtpProvider::build_mime()` is now public — reusable for custom providers
- TUI and CLI authenticate via SaaS API (JWT-based)
- TUI login screen with Ferrum branding

### Fixed
- Sending to @ferrum-mail.com users no longer routes through external SMTP (direct local delivery)
- Clippy + fmt cleanups across entire workspace

## [0.1.4] - 2026-04-01

### Added
- TUI login screen — authenticate via Ferrum Mail SaaS API
- SLSA release workflow now includes `ferrum-tui` binary with checksums
- `ferrum-email` umbrella crate added to release artifacts

### Fixed
- Export fixes, empty logo resolved
- CI clippy + fmt (collapsible_if, single_match, is_multiple_of)
- Unused variable warnings cleaned up

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
