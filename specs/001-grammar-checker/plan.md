# Implementation Plan: Grammar Checker Tool

**Branch**: `001-grammar-checker` | **Date**: 2026-01-06 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-grammar-checker/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Build a dual-interface grammar checking tool (Rust CLI + Chrome Extension) that operates in two modes: Local (Ollama/Llama 3) and Cloud (Gemini/OpenAI APIs). Both clients use a shared JSON Schema for LLM communication via a Provider pattern, returning edit suggestions as position-based diffs (start_index, end_index, replacement).

## Technical Context

### CLI Component
**Language/Version**: Rust stable, edition 2021+
**Primary Dependencies**: clap (CLI parsing), reqwest (HTTP), serde (JSON), keyring (OS credential storage), colored (terminal output), directories (config paths), toml (config format)
**Storage**: OS Keychain for API credentials; TOML config at `~/.config/grammar-check/config.toml` (XDG spec)
**Testing**: cargo test + integration tests with mock LLM responses
**Target Platform**: macOS, Windows, Linux

### Extension Component
**Language/Version**: TypeScript strict mode, ES2020+ target
**Primary Dependencies**: React, Vite, Zustand (state management per constitution)
**Storage**: Chrome extension storage API for config and encrypted credentials (Web Crypto API)
**Testing**: Vitest + React Testing Library
**Target Platform**: Chrome (Manifest V3)

### Shared
**Project Type**: Multi-platform (CLI + Browser Extension)
**Performance Goals**: CLI startup <100ms; Extension highlights <2s; Cloud check <10s for 1000 words; Local check <30s for 1000 words
**Constraints**: Local Mode = zero network calls; documents up to 50,000 chars; unicode-safe position handling
**Scale/Scope**: Single-user tool; English language only for v1

### Architecture
**Provider Pattern**: Abstract `Provider` trait/interface with `OllamaProvider` (localhost:11434) and `CloudProvider` (Gemini/OpenAI REST APIs)
**Shared Schema**: JSON Schema defining Edit objects `{start_index, end_index, replacement, category}` used by both clients
**Data Flow**: Client captures text → selects Provider from Config → POSTs text + system prompt → validates JSON response against Schema → renders diffs/highlights

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Principle I: Privacy First ✅
| Requirement | Status | Implementation |
|-------------|--------|----------------|
| Cloud API keys in OS Keychain | ✅ Planned | Rust: `keyring` crate; Extension: encrypted storage via Web Crypto API |
| Local Mode = zero network calls | ✅ Planned | Ollama provider uses localhost:11434 only; no telemetry |
| No telemetry without consent | ✅ Planned | No analytics code included |
| Temp files in secure directories | ✅ Planned | Use OS temp directories with user-scoped permissions |

### Principle II: Performance ✅
| Requirement | Status | Implementation |
|-------------|--------|----------------|
| CLI startup <100ms | ✅ Planned | Rust binary, lazy initialization, no heavy framework |
| Extension non-blocking | ✅ Planned | Web Workers for LLM calls; React concurrent features |
| UI responsive during analysis | ✅ Planned | Async operations with loading states |
| Bounded memory for large docs | ✅ Planned | Semantic chunking (3.5k chars, 400 char overlap) with bounded async concurrency (max 3) |

### Principle III: Type Safety ✅
| Requirement | Status | Implementation |
|-------------|--------|----------------|
| Rust: no `.unwrap()` in production | ✅ Planned | Use `?` operator and `Result<T, E>` types throughout |
| TypeScript: no `any` types | ✅ Planned | `strict: true` in tsconfig; use generics and `unknown` |
| Complete type signatures | ✅ Planned | All public APIs typed; JSON Schema → TypeScript types |
| Strict compiler settings | ✅ Planned | `#![deny(clippy::unwrap_used)]`; `strict: true` |

### Principle IV: Dependency Minimization ✅
| Requirement | Status | Implementation |
|-------------|--------|----------------|
| Rust: reqwest + serde only | ✅ Planned | clap, reqwest, serde, keyring, colored (all focused) |
| TypeScript: Zustand for state | ✅ Planned | No Redux/MobX; Zustand is lightweight |
| Evaluate bundle size | ✅ Planned | Vite tree-shaking; monitor extension size |
| Prefer std library | ✅ Planned | Use std::fs, std::io where possible |

**Gate Status**: ✅ PASS - All principles satisfied. Proceed to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/001-grammar-checker/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   └── llm-schema.json  # Shared JSON Schema for LLM communication
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
# CLI Component (Rust)
cli/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point, CLI argument parsing
│   ├── lib.rs               # Library exports for integration tests
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── check.rs         # `check` subcommand implementation
│   │   └── fix.rs           # `fix` subcommand implementation
│   ├── providers/
│   │   ├── mod.rs           # Provider trait definition
│   │   ├── ollama.rs        # OllamaProvider (Local Mode)
│   │   └── cloud.rs         # CloudProvider (Gemini/OpenAI)
│   ├── models/
│   │   ├── mod.rs
│   │   ├── edit.rs          # Edit struct matching JSON Schema
│   │   └── config.rs        # Configuration types
│   ├── services/
│   │   ├── mod.rs
│   │   ├── diff.rs          # Diff application logic
│   │   ├── chunker.rs       # Text chunking for large documents
│   │   └── validator.rs     # JSON response validation
│   └── output/
│       ├── mod.rs
│       ├── human.rs         # Colored terminal output
│       └── json.rs          # JSON output formatter
└── tests/
    ├── integration/
    │   ├── check_test.rs
    │   └── fix_test.rs
    └── fixtures/
        └── sample_texts/

# Extension Component (React + TypeScript)
extension/
├── package.json
├── vite.config.ts
├── tsconfig.json
├── manifest.json            # Chrome Manifest V3
├── src/
│   ├── background/
│   │   └── service-worker.ts  # Background script
│   ├── content/
│   │   ├── index.tsx          # Content script entry
│   │   ├── FloatingButton.tsx # Hover-activated button
│   │   ├── HighlightOverlay.tsx # Shadow DOM highlight renderer
│   │   └── FixPopup.tsx       # Inline fix suggestion popup
│   ├── popup/
│   │   └── Popup.tsx          # Extension popup (settings)
│   ├── providers/
│   │   ├── index.ts           # Provider interface
│   │   ├── ollama.ts          # OllamaProvider
│   │   └── cloud.ts           # CloudProvider
│   ├── store/
│   │   └── useGrammarStore.ts # Zustand store
│   ├── types/
│   │   └── edit.ts            # TypeScript types from JSON Schema
│   └── utils/
│       ├── diffApplier.ts     # Apply edits to text
│       └── positionMapper.ts  # Unicode-safe position handling
└── tests/
    ├── unit/
    └── integration/

# Shared Resources
shared/
└── schemas/
    └── llm-response.schema.json  # JSON Schema for LLM response format
```

**Structure Decision**: Multi-platform layout with separate `cli/` (Rust) and `extension/` (TypeScript/React) directories. A `shared/schemas/` directory contains the JSON Schema that defines the contract between clients and LLM providers. This structure keeps each platform independent while sharing the critical response format specification.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations. All principles satisfied.

---

## Post-Design Constitution Re-check

*Re-evaluated after Phase 1 design artifacts generated.*

### Principle I: Privacy First ✅ VERIFIED
- **CLI credentials**: `keyring` crate stores in OS Keychain (research.md §2)
- **Extension credentials**: Web Crypto API encrypted storage; no native messaging complexity (research.md §2)
- **Local Mode**: Ollama localhost only; zero external network calls verified in data-model.md

### Principle II: Performance ✅ VERIFIED
- **CLI startup**: Rust binary with lazy init; no framework overhead
- **Chunking**: 3.5k char semantic chunks with 400 char overlap; max 3 concurrent (research.md §6)
- **Extension**: Shadow DOM with CSS containment; Web Workers for LLM calls (research.md §5)

### Principle III: Type Safety ✅ VERIFIED
- **JSON Schema**: `contracts/llm-response.schema.json` defines strict types
- **Data Model**: All entities typed with Rust `#[derive(Serialize, Deserialize)]` and TypeScript interfaces
- **Validation**: Edit positions validated against document bounds (data-model.md)

### Principle IV: Dependency Minimization ✅ VERIFIED
- **CLI deps**: clap, reqwest, serde, keyring, colored, directories, toml, tokio - all focused crates
- **Extension deps**: React, Zustand only; no Redux/heavy frameworks
- **No new deps**: Research did not identify any additional dependencies needed

**Final Gate Status**: ✅ PASS - All principles verified post-design.

---

## Generated Artifacts

| Artifact | Path | Status |
|----------|------|--------|
| Research | `specs/001-grammar-checker/research.md` | ✅ Complete |
| Data Model | `specs/001-grammar-checker/data-model.md` | ✅ Complete |
| JSON Schema | `specs/001-grammar-checker/contracts/llm-response.schema.json` | ✅ Complete |
| Quickstart | `specs/001-grammar-checker/quickstart.md` | ✅ Complete |
| Agent Context | `CLAUDE.md` | ✅ Updated |

---

## Next Steps

Run `/speckit.tasks` to generate the task breakdown for implementation.
