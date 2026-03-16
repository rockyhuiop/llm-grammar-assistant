# Data Model: Grammar Checker Tool

**Feature**: 001-grammar-checker
**Date**: 2026-01-06
**Status**: Complete

This document defines the core entities, their relationships, validation rules, and state transitions for the grammar checker tool.

---

## Entity Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Configuration                                   │
│  ┌─────────────────┐                                                     │
│  │ OperatingMode   │                                                     │
│  │ ─────────────── │                                                     │
│  │ Local | Cloud   │                                                     │
│  └────────┬────────┘                                                     │
│           │                                                              │
│  ┌────────▼────────┐    ┌─────────────────┐                             │
│  │ CloudConfig     │    │ LocalConfig     │                             │
│  │ ─────────────── │    │ ─────────────── │                             │
│  │ provider        │    │ ollama_host     │                             │
│  │ model_name      │    │ model_name      │                             │
│  │ api_key_ref     │    └─────────────────┘                             │
│  └─────────────────┘                                                     │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│                           Grammar Check Flow                             │
│                                                                          │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐     │
│  │ CheckRequest    │───▶│ CheckResult     │───▶│ Edit            │     │
│  │ ─────────────── │    │ ─────────────── │    │ ─────────────── │     │
│  │ text            │    │ original_text   │    │ start_index     │     │
│  │ options         │    │ edits[]         │    │ end_index       │     │
│  └─────────────────┘    │ metadata        │    │ replacement     │     │
│                         └─────────────────┘    │ category        │     │
│                                                │ explanation?    │     │
│                                                └─────────────────┘     │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Core Entities

### Edit

Represents a single suggested correction. This is the fundamental unit returned by the LLM and processed by both CLI and Extension.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `start_index` | `integer` | Yes | Start position (UTF-16 code unit index, 0-based) |
| `end_index` | `integer` | Yes | End position (exclusive, UTF-16 code unit index) |
| `replacement` | `string` | Yes | Suggested replacement text |
| `category` | `enum` | Yes | Issue type: `"grammar"` or `"style"` |
| `explanation` | `string` | No | Optional reason for the correction |

**Validation Rules:**
- `start_index >= 0`
- `end_index > start_index`
- `end_index <= text.length` (within document bounds)
- `replacement` must differ from original text at that position
- Positions must not fall in the middle of a surrogate pair

**Rust Definition:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edit {
    pub start_index: usize,
    pub end_index: usize,
    pub replacement: String,
    pub category: EditCategory,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EditCategory {
    Grammar,
    Style,
}
```

**TypeScript Definition:**
```typescript
interface Edit {
  start_index: number;
  end_index: number;
  replacement: string;
  category: 'grammar' | 'style';
  explanation?: string;
}
```

---

### CheckResult

Collection of edits for a given text input with processing metadata.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `original_text` | `string` | Yes | The input text that was checked |
| `edits` | `Edit[]` | Yes | List of suggested edits (may be empty) |
| `metadata` | `ProcessingMetadata` | Yes | Processing information |

**Rust Definition:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub original_text: String,
    pub edits: Vec<Edit>,
    pub metadata: ProcessingMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingMetadata {
    pub mode: OperatingMode,
    pub provider: String,           // "ollama", "gemini", "openai"
    pub model: String,              // "llama3", "gpt-4o-mini", etc.
    pub processing_time_ms: u64,
    pub chunks_processed: Option<u32>,  // For large documents
}
```

---

### Configuration

User preferences and mode selection. Persisted across sessions.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `mode` | `OperatingMode` | Yes | Current operating mode |
| `local` | `LocalConfig` | No | Local mode settings (if configured) |
| `cloud` | `CloudConfig` | No | Cloud mode settings (if configured) |

**Rust Definition:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub mode: OperatingMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local: Option<LocalConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud: Option<CloudConfig>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OperatingMode {
    Local,
    Cloud,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    #[serde(default = "default_ollama_host")]
    pub ollama_host: String,   // default: "http://localhost:11434"
    #[serde(default = "default_local_model")]
    pub model_name: String,    // default: "llama3"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudConfig {
    pub provider: CloudProvider,
    #[serde(default)]
    pub model_name: Option<String>,  // provider-specific default if not set
    // Note: API key stored in OS keychain, referenced by provider name
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CloudProvider {
    Gemini,
    OpenAI,
}

fn default_ollama_host() -> String {
    "http://localhost:11434".to_string()
}

fn default_local_model() -> String {
    "llama3".to_string()
}
```

**TOML Example (config.toml):**
```toml
mode = "local"

[local]
ollama_host = "http://localhost:11434"
model_name = "llama3"

[cloud]
provider = "openai"
# API key retrieved from OS keychain using service name "grammar-check"
```

---

### CheckRequest (Internal)

Request structure used internally for grammar checking operations.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `text` | `string` | Yes | Text to check |
| `options` | `CheckOptions` | No | Processing options |

**Rust Definition:**
```rust
#[derive(Debug, Clone)]
pub struct CheckRequest {
    pub text: String,
    pub options: CheckOptions,
}

#[derive(Debug, Clone, Default)]
pub struct CheckOptions {
    pub output_format: OutputFormat,
    pub include_explanations: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum OutputFormat {
    #[default]
    Human,
    Json,
}
```

---

## State Transitions

### Configuration State Machine

```
                    ┌─────────────────┐
                    │   Unconfigured   │
                    │   (first run)    │
                    └────────┬────────┘
                             │
                    User selects mode
                             │
          ┌──────────────────┼──────────────────┐
          ▼                  │                  ▼
┌─────────────────┐          │        ┌─────────────────┐
│  Local Mode     │          │        │  Cloud Mode     │
│  Configured     │          │        │  Configured     │
└────────┬────────┘          │        └────────┬────────┘
         │                   │                 │
         │                   │    ┌────────────┴────────────┐
         │                   │    │                         │
         │                   │    ▼                         ▼
         │             ┌──────────────┐           ┌──────────────┐
         │             │ API Key      │           │ API Key      │
         │             │ Missing      │           │ Stored       │
         │             └──────┬───────┘           └──────────────┘
         │                    │
         │            User adds key
         │                    │
         ▼                    ▼
┌─────────────────────────────────────────────────┐
│                    Ready                         │
│  (can process grammar check requests)            │
└─────────────────────────────────────────────────┘
```

### Grammar Check Request Flow

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Pending    │────▶│  Processing  │────▶│  Completed   │
└──────────────┘     └──────┬───────┘     └──────────────┘
                           │
                           │ (error)
                           ▼
                    ┌──────────────┐
                    │   Failed     │
                    │  (retryable?)│
                    └──────────────┘
```

---

## Relationships

| From | To | Cardinality | Description |
|------|----|-------------|-------------|
| CheckResult | Edit | 1:N | A check result contains zero or more edits |
| Configuration | LocalConfig | 1:0..1 | Optional local mode configuration |
| Configuration | CloudConfig | 1:0..1 | Optional cloud mode configuration |
| CheckRequest | CheckResult | 1:1 | Each request produces exactly one result |

---

## Validation Rules Summary

### Edit Validation
```rust
impl Edit {
    pub fn validate(&self, text: &str) -> Result<(), ValidationError> {
        // Position bounds
        if self.start_index >= self.end_index {
            return Err(ValidationError::InvalidRange);
        }
        if self.end_index > text.encode_utf16().count() {
            return Err(ValidationError::OutOfBounds);
        }

        // Must be actual change
        let original: String = text.encode_utf16()
            .skip(self.start_index)
            .take(self.end_index - self.start_index)
            .map(|c| char::decode_utf16([c].iter().copied()))
            .flatten()
            .filter_map(|r| r.ok())
            .collect();

        if original == self.replacement {
            return Err(ValidationError::NoChange);
        }

        Ok(())
    }
}
```

### Configuration Validation
```rust
impl Configuration {
    pub fn validate(&self) -> Result<(), ConfigError> {
        match self.mode {
            OperatingMode::Local => {
                if self.local.is_none() {
                    return Err(ConfigError::MissingLocalConfig);
                }
            }
            OperatingMode::Cloud => {
                if self.cloud.is_none() {
                    return Err(ConfigError::MissingCloudConfig);
                }
            }
        }
        Ok(())
    }
}
```

---

## Extension-Specific Types

The browser extension uses TypeScript versions of these types:

```typescript
// Extension state (Zustand store)
interface GrammarStore {
  // Current state
  isChecking: boolean;
  currentTextareaId: string | null;
  edits: Edit[];

  // Configuration
  config: ExtensionConfig;

  // Actions
  checkText: (text: string) => Promise<void>;
  applyEdit: (edit: Edit) => void;
  clearEdits: () => void;
}

interface ExtensionConfig {
  mode: 'local' | 'cloud';
  cloudProvider?: 'gemini' | 'openai';
  localHost?: string;  // default: "http://localhost:11434"
}
```

---

## Chunking Types (Internal)

For processing large documents (>50k characters):

```rust
#[derive(Debug, Clone)]
pub struct Chunk {
    pub index: usize,
    pub text: String,
    pub original_start: usize,      // Position in full document
    pub overlap_before: usize,      // Chars of overlap with previous
    pub overlap_after: usize,       // Chars of overlap with next
    pub primary_range: Range<usize>, // Valid region for edit reporting
}

#[derive(Debug, Clone)]
pub struct ChunkingOptions {
    pub target_size: usize,         // default: 3500
    pub overlap_size: usize,        // default: 400
    pub boundary_strategy: BoundaryStrategy,
}

#[derive(Debug, Clone, Copy)]
pub enum BoundaryStrategy {
    Paragraph,  // Split at double newlines
    Sentence,   // Split at sentence endings
    Flexible,   // Prefer paragraph, fallback to sentence
}
```
