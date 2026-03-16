# Tasks: Grammar Checker Tool

**Feature**: 001-grammar-checker
**Input**: Design documents from `/specs/001-grammar-checker/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: User story label (US1-US5) - only for story phase tasks
- Exact file paths included in descriptions

## Path Conventions

This project uses a **multi-platform layout**:
- **CLI (Rust)**: `cli/src/`
- **Extension (TypeScript)**: `extension/src/`
- **Shared**: `shared/schemas/`

---

## Phase 1: Setup (Project Initialization)

**Purpose**: Initialize both CLI and Extension projects with correct structure

- [X] T001 Create root directory structure: `cli/`, `extension/`, `shared/schemas/`
- [X] T002 [P] Copy JSON Schema from `specs/001-grammar-checker/contracts/llm-response.schema.json` to `shared/schemas/llm-response.schema.json`
- [X] T003 [P] Initialize Rust project with `cargo init` in `cli/`
- [X] T004 [P] Initialize TypeScript/Vite project with `pnpm create vite` in `extension/`
- [X] T005 Configure `cli/Cargo.toml` with dependencies per research.md (clap, reqwest, serde, keyring, colored, directories, toml, tokio, unicode-segmentation, anyhow, thiserror)
- [X] T006 [P] Configure `extension/package.json` with dependencies per research.md (react, zustand, vite, typescript, @types/chrome)
- [X] T007 [P] Configure `extension/tsconfig.json` with `strict: true` per constitution
- [X] T008 [P] Create `extension/manifest.json` for Chrome Manifest V3
- [X] T009 [P] Configure `extension/vite.config.ts` for extension build output
- [X] T010 Add `#![deny(clippy::unwrap_used)]` lint to `cli/src/lib.rs` per constitution

**Checkpoint**: Both projects scaffold complete, can run `cargo check` and `pnpm build`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared infrastructure that MUST be complete before ANY user story

**CRITICAL**: No user story work can begin until this phase is complete. Configuration (US5) is extracted here as it's needed by all stories.

### Core Types (Shared between CLI and Extension)

- [X] T011 [P] Create Edit struct in `cli/src/models/edit.rs` matching JSON Schema
- [X] T012 [P] Create EditCategory enum (Grammar, Style) in `cli/src/models/edit.rs`
- [X] T013 [P] Create CheckResult struct in `cli/src/models/edit.rs`
- [X] T014 [P] Create ProcessingMetadata struct in `cli/src/models/edit.rs`
- [X] T015 [P] Create TypeScript Edit interface in `extension/src/types/edit.ts` matching JSON Schema
- [X] T016 [P] Create TypeScript CheckResult interface in `extension/src/types/edit.ts`

### Configuration System (from US5 - foundational)

- [X] T017 [P] Create Configuration struct in `cli/src/models/config.rs` per data-model.md
- [X] T018 [P] Create OperatingMode enum (Local, Cloud) in `cli/src/models/config.rs`
- [X] T019 [P] Create LocalConfig struct in `cli/src/models/config.rs`
- [X] T020 [P] Create CloudConfig struct with CloudProvider enum in `cli/src/models/config.rs`
- [X] T021 Implement config loading from TOML file using `directories` crate in `cli/src/models/config.rs`
- [X] T022 Implement config persistence (save to TOML file) in `cli/src/models/config.rs`
- [X] T023 [P] Create ExtensionConfig interface in `extension/src/types/config.ts`
- [X] T024 Implement chrome.storage.local wrapper for config in `extension/src/utils/storage.ts`

### Provider Infrastructure

- [X] T025 Create Provider trait definition in `cli/src/providers/mod.rs` with `async fn check(&self, text: &str) -> Result<Vec<Edit>>`
- [X] T026 [P] Create Provider interface in `extension/src/providers/index.ts`

### LLM System Prompt

- [X] T027 Create grammar check system prompt template in `cli/src/providers/prompts.rs` (instructs LLM to return JSON matching schema)

### Module Structure

- [X] T028 Create `cli/src/models/mod.rs` exporting edit and config modules
- [X] T029 [P] Create `cli/src/providers/mod.rs` with Provider trait re-export
- [X] T030 [P] Create `cli/src/services/mod.rs` module structure
- [X] T031 [P] Create `cli/src/output/mod.rs` module structure
- [X] T032 [P] Create `cli/src/commands/mod.rs` module structure
- [X] T033 Create `cli/src/lib.rs` with all module declarations

**Checkpoint**: Foundation ready - can create provider implementations and commands

---

## Phase 3: User Story 1 - CLI Grammar Check (Priority: P1) - MVP

**Goal**: Writer can check document for grammar errors from CLI, see suggested corrections with context

**Independent Test**: Run `echo "He dont like it" | grammar-check check` and verify output shows "dont" → "doesn't" suggestion with position info

### Provider Implementations for US1

- [X] T034 [US1] Implement OllamaProvider in `cli/src/providers/ollama.rs` with /api/chat endpoint per research.md
- [X] T035 [US1] Add Ollama health check (verify localhost:11434 responds) in `cli/src/providers/ollama.rs`
- [X] T036 [US1] Implement OpenAI CloudProvider in `cli/src/providers/cloud.rs` with JSON schema response format
- [X] T037 [US1] Implement Gemini CloudProvider in `cli/src/providers/cloud.rs` with responseSchema
- [X] T038 [US1] Add exponential backoff retry logic (3 attempts) in `cli/src/providers/cloud.rs` per research.md

### Core Services for US1

- [X] T039 [US1] Implement JSON response validator in `cli/src/services/validator.rs` (validate edit positions within bounds)
- [X] T040 [US1] Implement text chunker in `cli/src/services/chunker.rs` per research.md (3.5k chars, 400 overlap, semantic boundaries)
- [X] T041 [US1] Implement chunk position merging in `cli/src/services/chunker.rs` (offset edits, deduplicate)
- [X] T042 [US1] Implement Unicode position handling in `cli/src/services/position.rs` (UTF-16 code unit indices)

### Output Formatters for US1

- [X] T043 [P] [US1] Implement human-readable output formatter in `cli/src/output/human.rs` (colored, with context)
- [X] T044 [P] [US1] Implement JSON output formatter in `cli/src/output/json.rs`

### Check Command Implementation

- [X] T045 [US1] Implement stdin text reader in `cli/src/commands/check.rs`
- [X] T046 [US1] Implement file path text reader in `cli/src/commands/check.rs`
- [X] T047 [US1] Implement `check` subcommand in `cli/src/commands/check.rs` with --format flag (human/json)
- [X] T048 [US1] Handle empty input case ("No text to check" message) in `cli/src/commands/check.rs`
- [X] T049 [US1] Handle "no issues found" case (exit code 0) in `cli/src/commands/check.rs`

### CLI Entry Point

- [X] T050 [US1] Create main.rs with clap CLI parser in `cli/src/main.rs`
- [X] T051 [US1] Add first-run blocking mode prompt (if no config exists) in `cli/src/main.rs`
- [X] T052 [US1] Wire check command with provider selection based on config in `cli/src/main.rs`

### Keyring Integration (for Cloud Mode)

- [X] T053 [US1] Implement API key storage/retrieval using `keyring` crate in `cli/src/services/credentials.rs`
- [X] T054 [US1] Add `config --set-api-key` command for storing credentials in `cli/src/commands/config.rs`

**Checkpoint**: CLI `grammar-check check` works with both Local (Ollama) and Cloud modes. User Story 1 complete.

---

## Phase 4: User Story 2 - CLI Grammar Fix (Priority: P2)

**Goal**: Writer can automatically apply all grammar corrections to document

**Independent Test**: Create file with "He dont like it", run `grammar-check fix input.txt`, verify output is "He doesn't like it"

### Diff Application Service

- [X] T055 [US2] Implement diff applier in `cli/src/services/diff.rs` (apply edits in reverse order to preserve positions)
- [X] T056 [US2] Handle overlapping edits (consistent ordering) in `cli/src/services/diff.rs`

### Fix Command Implementation

- [X] T057 [US2] Implement `fix` subcommand in `cli/src/commands/fix.rs` (outputs corrected text to stdout)
- [X] T058 [US2] Add --output flag for writing to file in `cli/src/commands/fix.rs`
- [X] T059 [US2] Add --in-place flag for modifying input file directly in `cli/src/commands/fix.rs`
- [X] T060 [US2] Wire fix command in `cli/src/main.rs`

**Checkpoint**: CLI `grammar-check fix` works with all output modes. User Stories 1+2 complete.

---

## Phase 5: User Story 3 - Browser Extension Highlighting (Priority: P3)

**Goal**: User sees grammar errors highlighted in red and style issues in yellow directly in web textareas

**Independent Test**: Load extension, type "He dont like it" in any textarea, click floating button, verify red highlight on "dont"

### Extension Provider Implementations

- [X] T061 [P] [US3] Implement OllamaProvider in `extension/src/providers/ollama.ts` mirroring CLI implementation
- [X] T062 [P] [US3] Implement CloudProvider (OpenAI/Gemini) in `extension/src/providers/cloud.ts`
- [X] T063 [US3] Add provider factory based on config in `extension/src/providers/index.ts`

### Extension State Management

- [X] T064 [US3] Create Zustand store in `extension/src/store/useGrammarStore.ts` per data-model.md
- [X] T065 [US3] Implement checkText action in store (calls provider, stores edits)
- [X] T066 [US3] Implement clearEdits action in store
- [X] T066a [US3] Handle empty textarea case in store (show "No text to check" or skip analysis)

### Content Script Infrastructure

- [X] T067 [US3] Create content script entry point in `extension/src/content/index.tsx`
- [X] T068 [US3] Implement textarea detection (find all textareas on page) in `extension/src/content/index.tsx`
- [X] T069 [US3] Implement MutationObserver for dynamically created textareas in `extension/src/content/index.tsx`

### Floating Button Component

- [X] T070 [US3] Create FloatingButton component in `extension/src/content/FloatingButton.tsx`
- [X] T071 [US3] Implement hover detection (show button when near textarea) in FloatingButton
- [X] T072 [US3] Implement fade-in/fade-out animation in FloatingButton

### Shadow DOM Highlight Overlay

- [X] T073 [US3] Create GrammarHighlightOverlay custom element in `extension/src/content/HighlightOverlay.tsx` per research.md
- [X] T074 [US3] Implement closed Shadow DOM with CSS isolation (`:host { all: initial }`) in HighlightOverlay
- [X] T075 [US3] Implement textarea metrics sampling (font, padding, line-height) in HighlightOverlay
- [X] T076 [US3] Implement character position to pixel position mapping in `extension/src/utils/positionMapper.ts`
- [X] T077 [US3] Implement highlight rendering (red #FF6B6B for grammar, yellow #FFE066 for style) in HighlightOverlay
- [X] T078 [US3] Implement ResizeObserver for dynamic textarea resizing in HighlightOverlay
- [X] T079 [US3] Implement scroll synchronization in HighlightOverlay
- [X] T079a [US3] Handle text change during analysis: cancel pending request and clear stale highlights in HighlightOverlay

### Tooltip Component

- [X] T080 [US3] Implement hover tooltip showing suggested replacement in HighlightOverlay

### Error State Handling

- [X] T080a [US3] Display error state in FloatingButton when provider fails (red icon + tooltip with error message)
- [X] T080b [US3] Handle retry exhaustion gracefully (show "Service unavailable" after 3 failed attempts)

### Background Service Worker

- [X] T081 [US3] Create service worker in `extension/src/background/service-worker.ts`
- [X] T082 [US3] Handle config loading from chrome.storage in service worker

**Checkpoint**: Extension shows floating button and highlights errors. User Story 3 complete.

---

## Phase 6: User Story 4 - Browser Extension Quick Fix (Priority: P4)

**Goal**: User can click a highlighted error and apply the fix with one click

**Independent Test**: Click red highlight on "dont", see popup with "doesn't" suggestion, click Apply, verify textarea updates

### Fix Popup Component

- [X] T083 [US4] Create FixPopup component in `extension/src/content/FixPopup.tsx`
- [X] T084 [US4] Show original text and suggested replacement in popup
- [X] T085 [US4] Implement "Apply" button click handler in FixPopup

### Diff Application

- [X] T086 [US4] Implement diffApplier utility in `extension/src/utils/diffApplier.ts` (apply single edit to textarea value)
- [X] T087 [US4] Update textarea content when fix applied in diffApplier
- [X] T088 [US4] Implement applyEdit action in Zustand store

### Position Adjustment

- [X] T089 [US4] Recalculate remaining highlight positions after fix applied in store
- [X] T090 [US4] Remove applied highlight from overlay

**Checkpoint**: Extension allows one-click fixes. User Stories 3+4 complete.

---

## Phase 7: User Story 5 - Mode Selection and Configuration (Priority: P5)

**Goal**: User can configure Local/Cloud mode and credentials that persist across sessions

**Note**: Core configuration was implemented in Phase 2 (Foundational). This phase adds the UI and complete UX.

**Independent Test**: Open extension popup, select Cloud mode, enter API key, reload extension, verify setting persists

### Extension Popup UI

- [X] T091 [US5] Create Popup component in `extension/src/popup/Popup.tsx`
- [X] T092 [US5] Implement mode selector (Local/Cloud radio buttons) in Popup
- [X] T093 [US5] Implement cloud provider selector (Gemini/OpenAI dropdown) in Popup
- [X] T094 [US5] Implement API key input field with show/hide toggle in Popup

### Credential Encryption

- [X] T095 [US5] Implement Web Crypto API encryption for API keys in `extension/src/utils/crypto.ts`
- [X] T096 [US5] Store encrypted API key in chrome.storage.local in crypto.ts

### CLI Config Command

- [X] T097 [US5] Implement `config --mode <local|cloud>` command in `cli/src/commands/config.rs`
- [X] T098 [US5] Implement `config --provider <gemini|openai>` command in `cli/src/commands/config.rs`
- [X] T099 [US5] Implement `config --show` command to display current settings in `cli/src/commands/config.rs`

**Checkpoint**: Configuration complete for both CLI and Extension. All user stories complete.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Final improvements affecting multiple user stories

- [X] T100 [P] Add comprehensive error messages for all failure modes in `cli/src/main.rs`
- [X] T101 [P] Add loading spinner/indicator during grammar check in `extension/src/content/FloatingButton.tsx`
- [X] T102 [P] Add keyboard shortcut support (optional) in extension
- [X] T103 Verify CLI startup time <100ms with `time grammar-check --help`
- [X] T104 Verify extension bundle size is reasonable (<500KB) with `pnpm build`
- [X] T105 Run quickstart.md validation (test all documented commands)
- [X] T106 [P] Add sample test fixtures in `cli/tests/fixtures/sample_texts/`

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 1: Setup → No dependencies (start immediately)
    │
    ▼
Phase 2: Foundational → BLOCKS all user stories
    │
    ├──────────────────────────────────────────┐
    │                                          │
    ▼                                          ▼
Phase 3: US1 (CLI Check)              Phase 5: US3 (Ext Highlight)
    │                                          │
    ▼                                          ▼
Phase 4: US2 (CLI Fix)                Phase 6: US4 (Ext Fix)
    │                                          │
    └──────────────────────────────────────────┘
                        │
                        ▼
              Phase 7: US5 (Config UI)
                        │
                        ▼
              Phase 8: Polish
```

### User Story Dependencies

| Story | Depends On | Can Parallel With |
|-------|------------|-------------------|
| US1 (CLI Check) | Phase 2 | US3 (different platform) |
| US2 (CLI Fix) | US1 | US3, US4 (different platform) |
| US3 (Ext Highlight) | Phase 2 | US1, US2 (different platform) |
| US4 (Ext Fix) | US3 | US2 (different platform) |
| US5 (Config UI) | US1 or US3 | - |

### Within Each Phase

- Tasks marked [P] can run in parallel
- Models before services
- Services before commands/components
- Core implementation before integration

---

## Parallel Execution Examples

### Phase 2 Parallel Opportunities

```bash
# Launch all type definitions in parallel:
T011: Create Edit struct in cli/src/models/edit.rs
T012: Create EditCategory enum in cli/src/models/edit.rs
T015: Create TypeScript Edit interface in extension/src/types/edit.ts

# Launch all config types in parallel:
T017: Create Configuration struct in cli/src/models/config.rs
T023: Create ExtensionConfig interface in extension/src/types/config.ts
```

### Cross-Platform Parallel Development

```bash
# CLI Team (Rust):
Phase 3: T034-T054 (US1 - CLI Check)
Phase 4: T055-T060 (US2 - CLI Fix)

# Extension Team (TypeScript) - IN PARALLEL:
Phase 5: T061-T082 (US3 - Extension Highlighting)
Phase 6: T083-T090 (US4 - Extension Quick Fix)
```

### User Story 1 Internal Parallelism

```bash
# Launch providers in parallel:
T034: Implement OllamaProvider in cli/src/providers/ollama.rs
T036: Implement OpenAI CloudProvider in cli/src/providers/cloud.rs

# Launch output formatters in parallel:
T043: Implement human-readable output in cli/src/output/human.rs
T044: Implement JSON output in cli/src/output/json.rs
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL)
3. Complete Phase 3: User Story 1 (CLI Check)
4. **STOP and VALIDATE**: Test with `echo "He dont like it" | grammar-check check`
5. Deploy/demo CLI check capability

### Incremental Delivery

| Milestone | Stories Complete | Value Delivered |
|-----------|------------------|-----------------|
| MVP | US1 | CLI grammar checking |
| CLI Complete | US1 + US2 | CLI check + fix |
| Extension MVP | US1 + US3 | CLI + Extension highlighting |
| Full Feature | US1-US5 | Complete tool |

### Single Developer Path

```
Setup → Foundational → US1 → US2 → US3 → US4 → US5 → Polish
```

### Two Developer Path

```
Developer A (CLI):     Setup → Foundational → US1 → US2 → US5 (CLI) → Polish
Developer B (Extension): (wait for Foundational) → US3 → US4 → US5 (Ext) → Polish
```

---

## Summary

| Metric | Count |
|--------|-------|
| **Total Tasks** | 110 |
| **Phase 1 (Setup)** | 10 |
| **Phase 2 (Foundational)** | 23 |
| **Phase 3 (US1 - CLI Check)** | 21 |
| **Phase 4 (US2 - CLI Fix)** | 6 |
| **Phase 5 (US3 - Ext Highlight)** | 26 |
| **Phase 6 (US4 - Ext Fix)** | 8 |
| **Phase 7 (US5 - Config UI)** | 9 |
| **Phase 8 (Polish)** | 7 |
| **Parallel Opportunities** | 38 tasks marked [P] |
| **MVP Scope** | 54 tasks (Phases 1-3) |

---

## Notes

- All tasks include exact file paths for immediate execution
- [P] tasks can be parallelized (different files, no dependencies)
- [USn] labels map tasks to user stories for traceability
- Tests not included (not explicitly requested in spec)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
