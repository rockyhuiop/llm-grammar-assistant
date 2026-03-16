# Research: Grammar Checker Tool

**Feature**: 001-grammar-checker
**Date**: 2026-01-06
**Status**: Complete

This document consolidates research findings to resolve all NEEDS CLARIFICATION items from the plan.md and establish best practices for implementation.

---

## 1. CLI Configuration File Location

### Decision
Use the `directories` crate with TOML configuration format stored at platform-specific locations.

### Rationale
- `directories::ProjectDirs::from()` automatically handles platform differences (XDG spec on Linux, `~/Library` on macOS, `%APPDATA%` on Windows)
- TOML is human-readable, strongly typed with serde, and the Rust ecosystem standard (Cargo.toml)
- Separates non-sensitive config from credentials (stored in OS keychain)

### Implementation
```rust
use directories::ProjectDirs;

// Config paths by platform:
// macOS:  ~/Library/Application Support/grammar-check/config.toml
// Linux:  ~/.config/grammar-check/config.toml
// Windows: %APPDATA%\grammar-check\config.toml

if let Some(proj_dirs) = ProjectDirs::from("", "", "grammar-check") {
    let config_path = proj_dirs.config_dir().join("config.toml");
}
```

### Alternatives Considered
| Alternative | Rejected Because |
|-------------|------------------|
| Custom path logic | Error-prone, duplicates platform logic |
| Environment variables only | No persistence, harder for users |
| JSON config | Less readable, stricter formatting |

---

## 2. Extension Credential Storage

### Decision
**DO NOT use native messaging for keychain access.** Store non-sensitive config in `chrome.storage.local`. API credentials are entered directly in the extension popup and stored encrypted in extension storage.

### Rationale
1. Native messaging adds deployment complexity (separate binary installation)
2. Manifest V3 Service Workers make persistent native messaging connections unreliable
3. The CLI handles local Ollama mode (no credentials needed); Cloud mode credentials in the extension are isolated to that context
4. For privacy-first principle: Local Mode uses no credentials; Cloud Mode credentials are per-interface (CLI uses keyring, Extension uses chrome.storage)

### Implementation
```typescript
// Extension settings storage
interface ExtensionConfig {
  mode: 'local' | 'cloud';
  cloudProvider?: 'gemini' | 'openai';
  // API key stored encrypted via Web Crypto API
  encryptedApiKey?: string;
}

// Store in chrome.storage.local (not sync for privacy)
chrome.storage.local.set({ config: configObject });
```

### Security Measures
- Encrypt API keys at rest using Web Crypto API with a device-derived key
- Never log or expose API keys in console output
- Clear credentials on extension uninstall

### Alternatives Considered
| Alternative | Rejected Because |
|-------------|------------------|
| Native messaging to CLI keyring | Complex deployment, 4 IPC layers |
| Backend proxy server | Violates privacy-first principle |
| Plain text in chrome.storage | Insecure |

---

## 3. Ollama API Implementation

### Decision
Use `/api/chat` endpoint with non-streaming mode, Llama 3 (8B) model, and post-response JSON validation.

### Rationale
- Chat endpoint supports system prompts for format enforcement
- Non-streaming ensures complete JSON responses for reliable parsing
- Llama 3 8B balances accuracy and performance for local grammar checking
- Low temperature (0.1) provides deterministic output

### Implementation
```rust
// POST http://localhost:11434/api/chat
{
  "model": "llama3",
  "messages": [
    {"role": "system", "content": "You are a grammar checker. Return JSON: {\"edits\": [...]}"},
    {"role": "user", "content": "Check: <user text>"}
  ],
  "stream": false,
  "temperature": 0.1
}
```

### Health Check
```rust
// Before processing, verify Ollama is running
let health = reqwest::get("http://localhost:11434").await?;
if !health.status().is_success() {
    return Err(Error::OllamaUnavailable);
}
```

---

## 4. Cloud API Implementation (Gemini/OpenAI)

### Decision
Dual-provider strategy with common interface. Both providers support JSON schema constraints natively.

### Rationale
- OpenAI has mature rate limit headers and stable API
- Gemini offers cost optimization with competitive pricing
- Both support `response_format` / `responseSchema` for structured output
- Abstraction layer enables provider switching without code changes

### Implementation

**OpenAI Request:**
```javascript
POST https://api.openai.com/v1/chat/completions
{
  "model": "gpt-4o-mini",
  "messages": [...],
  "response_format": {
    "type": "json_schema",
    "json_schema": { "name": "grammar_check", "schema": {...} }
  }
}
// Headers: Authorization: Bearer <key>
```

**Gemini Request:**
```javascript
POST https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash/generateContent?key=<key>
{
  "contents": [...],
  "generationConfig": {
    "responseMimeType": "application/json",
    "responseSchema": {...}
  }
}
```

### Retry Logic
- Max 3 retries with exponential backoff (1s, 2s, 4s base + jitter)
- Retry on: 429 (rate limit), 502/503 (gateway errors)
- Fail on: 401 (auth), 400 (bad request)

---

## 5. Shadow DOM Implementation for Highlights

### Decision
Use closed Shadow DOM with custom element wrapper hosting absolutely-positioned highlight divs.

### Rationale
- Closed mode prevents page scripts from accessing overlay internals
- `:host { all: initial }` prevents CSS cascade pollution
- `contain: layout style paint` isolates rendering
- Custom element (`<grammar-highlight-overlay>`) maintains semantic encapsulation

### Implementation
```typescript
class GrammarHighlightOverlay extends HTMLElement {
  constructor() {
    super();
    this.shadow = this.attachShadow({ mode: 'closed' });
    // Initialize isolated styles with CSS reset
  }
}
customElements.define('grammar-highlight-overlay', GrammarHighlightOverlay);
```

### Key CSS
```css
:host {
  all: initial;
  position: absolute;
  contain: layout style paint;
  pointer-events: none;
}
.highlight.grammar { background-color: #FF6B6B; opacity: 0.4; }
.highlight.style { background-color: #FFE066; opacity: 0.4; }
```

### Textarea Tracking
- Use `ResizeObserver` for dynamic resizing
- Listen to `scroll` events for highlight repositioning
- Sample textarea metrics (font, padding) at render time for accurate positioning

---

## 6. Text Chunking Strategy

### Decision
Semantic-aware chunking with 3,500 character chunks, 400-character symmetric overlap, and asynchronous processing.

### Rationale
- 3,500 chars (~700 words) provides sufficient context for grammar rules spanning clauses
- 400-char overlap captures errors at sentence boundaries
- Semantic boundaries (paragraph/sentence endings) preserve grammatical context
- Async processing with bounded concurrency (max 3) balances performance and API limits

### Algorithm
```
1. Split at semantic boundaries (paragraphs > sentences)
2. Target 3,500 chars per chunk; flex 50%-150%
3. Extend each chunk 400 chars backward/forward (overlap)
4. Process chunks with max 3 concurrent requests
5. Filter edits to "primary region" (non-overlap zone)
6. Offset positions by chunk's document position
7. Deduplicate by (start_index, end_index)
8. Sort by start_index
```

### Memory Efficiency
- Use `tokio::sync::mpsc::channel(3)` for bounded buffer
- Process results immediately; don't accumulate raw text
- For very large documents (>200k chars), stream chunks to avoid memory pressure

---

## 7. Unicode-Safe Position Handling

### Decision
Use **UTF-16 code unit indices** (JavaScript string positions) as the canonical representation in JSON Schema.

### Rationale
- Aligns with browser extension's natural indexing (textarea API uses JS string positions)
- LLM receives original text; its position outputs naturally fit this coordinate system
- Deterministic conversion to/from Rust byte offsets
- `String.prototype.slice(start, end)` works directly with these indices

### Position Conversion
```rust
// Rust: byte offset → JS index
fn byte_to_js_index(text: &str, byte_offset: usize) -> usize {
    let mut js_index = 0;
    for (byte_pos, c) in text.char_indices() {
        if byte_pos >= byte_offset { break; }
        js_index += if c as u32 > 0xFFFF { 2 } else { 1 };
    }
    js_index
}
```

### Edge Cases
| Case | Handling |
|------|----------|
| ZWJ sequences (family emoji) | Validate edits span complete sequences |
| Combining diacritics | Use `unicode-segmentation` for grapheme validation |
| CRLF line endings | Normalize to LF before processing |
| Lone surrogates | Sanitize input; reject invalid UTF-16 |

### JSON Schema Documentation
```json
{
  "$comment": "Positions are UTF-16 code unit indices (same as JavaScript string indices)"
}
```

---

## Resolved NEEDS CLARIFICATION Items

| Item | Resolution |
|------|------------|
| Config file location | `directories` crate: `~/.config/grammar-check/config.toml` (Linux), platform-specific |
| Extension credential access | Chrome extension storage with encryption; no native messaging |
| Streaming approach for large docs | Async chunking with bounded concurrency (max 3 parallel) |

---

## Dependencies Summary

### CLI (Rust)
```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
keyring = "3.0"
colored = "2.1"
directories = "5.0"
tokio = { version = "1.40", features = ["full"] }
unicode-segmentation = "1.11"
anyhow = "1.0"
thiserror = "1.0"
```

### Extension (TypeScript)
```json
{
  "dependencies": {
    "react": "^18.3",
    "zustand": "^4.5"
  },
  "devDependencies": {
    "typescript": "^5.5",
    "vite": "^5.4",
    "@types/chrome": "^0.0.270"
  }
}
```
