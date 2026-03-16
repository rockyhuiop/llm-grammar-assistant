<!--
================================================================================
SYNC IMPACT REPORT
================================================================================
Version Change: N/A → 1.0.0 (initial ratification)
Modified Principles: N/A (initial creation)
Added Sections:
  - Core Principles (4 principles)
  - Technical Constraints
  - Development Workflow
  - Governance
Removed Sections: N/A
Templates Requiring Updates:
  - .specify/templates/plan-template.md: ✅ Compatible (Constitution Check section exists)
  - .specify/templates/spec-template.md: ✅ Compatible (no constitution-specific sections)
  - .specify/templates/tasks-template.md: ✅ Compatible (no constitution-specific sections)
Follow-up TODOs: None
================================================================================
-->

# LLM Grammar Assistant Constitution

## Core Principles

### I. Privacy First

User data MUST never leave the device when operating in "Local Mode". This principle is
non-negotiable and applies to all data processing, caching, and temporary storage.

**Requirements**:
- All Cloud API keys MUST be stored in the OS Keychain (macOS Keychain, Windows Credential
  Manager, Linux Secret Service), never in plain text configuration files
- Local Mode MUST function entirely offline with zero network calls
- No telemetry, analytics, or crash reporting in Local Mode without explicit user consent
- Temporary files containing user data MUST be stored in secure, user-scoped directories

**Rationale**: Users trust grammar assistants with sensitive documents. Privacy violations
erode trust and may expose confidential information. OS-level secret storage provides
hardware-backed protection on supported platforms.

### II. Performance

The system MUST meet strict responsiveness targets to ensure a seamless user experience.

**Requirements**:
- CLI startup time MUST be under 100ms (measured from invocation to ready state)
- VS Code Extension MUST NOT block the main thread; all LLM calls and heavy processing
  MUST execute on background threads or Web Workers
- UI updates MUST remain responsive during grammar analysis (no freezing)
- Memory usage MUST remain bounded; large documents MUST be processed in streaming fashion

**Rationale**: Slow tools interrupt creative flow. A 100ms CLI startup ensures the tool
feels instant. Non-blocking extensions prevent VS Code from becoming unresponsive during
grammar checks.

### III. Type Safety

Strict typing MUST be enforced across all production code to catch errors at compile time.

**Requirements**:
- Rust code MUST NOT use `.unwrap()` in production paths; use `?` operator or explicit
  error handling with `Result` types
- TypeScript code MUST NOT use `any` type; use proper generics, union types, or `unknown`
  with type guards
- All public APIs MUST have complete type signatures
- Enable strictest compiler settings: `strict: true` in tsconfig, `#![deny(clippy::unwrap_used)]`
  in Rust

**Rationale**: Runtime panics from `.unwrap()` crash the application without recovery.
`any` types disable TypeScript's type checker, allowing bugs to slip through. Strict
typing catches errors during development rather than in production.

### IV. Dependency Minimization

Prefer simple, focused libraries over heavy frameworks to reduce attack surface, build
times, and maintenance burden.

**Requirements**:
- Rust: Use `reqwest` for HTTP and `serde` for serialization; avoid framework-heavy crates
- TypeScript/React: Use `Zustand` for state management instead of Redux or MobX
- Evaluate each dependency for: bundle size impact, maintenance activity, security history
- Prefer standard library solutions when performance difference is negligible

**Rationale**: Every dependency is a potential security vulnerability, license concern,
and maintenance burden. Lighter dependencies mean faster builds, smaller binaries, and
fewer breaking changes to track.

## Technical Constraints

**Language & Runtime Requirements**:
- Rust: Stable toolchain, edition 2021 or later
- TypeScript: Strict mode enabled, ES2020+ target
- Node.js: LTS versions only for extension development

**Security Boundaries**:
- All external input (user text, API responses) MUST be validated before processing
- File system access MUST be scoped to user-approved directories
- Network requests MUST use TLS 1.2+ exclusively

**Platform Support**:
- macOS, Windows, Linux for CLI
- Chrome (Manifest V3) for Browser Extension

## Development Workflow

**Code Quality Gates**:
- All PRs MUST pass type checking with zero errors
- All PRs MUST pass linting (clippy for Rust, ESLint for TypeScript)
- Performance-critical paths MUST include benchmark tests
- Security-sensitive changes MUST be reviewed by a second contributor

**Testing Requirements**:
- Unit tests for business logic
- Integration tests for API boundaries
- Manual testing for UI/UX changes in extension

## Governance

This constitution defines the non-negotiable principles for the LLM Grammar Assistant
project. All contributors, PRs, and architectural decisions MUST comply with these
principles.

**Amendment Process**:
1. Propose amendment via PR with rationale
2. Document impact on existing code
3. Obtain approval from project maintainers
4. Update version according to semantic versioning:
   - MAJOR: Principle removal or incompatible redefinition
   - MINOR: New principle or material expansion
   - PATCH: Clarifications and wording improvements

**Compliance Review**:
- All PRs MUST verify compliance with applicable principles
- Complexity or principle violations MUST be justified in writing
- Unjustified violations MUST be rejected

**Version**: 1.0.0 | **Ratified**: 2026-01-06 | **Last Amended**: 2026-01-06
