# Grammar Check

A grammar and style checker powered by LLMs, available as both a **Rust CLI** and a **Chrome Extension**. Operates in two modes:

- **Local Mode** — Uses [Ollama](https://ollama.ai) (Llama 3) running on your machine. Zero network calls, full privacy.
- **Cloud Mode** — Uses OpenAI (GPT-4o mini) or Google Gemini APIs for higher accuracy.

Both interfaces share a common JSON Schema contract for LLM communication and return position-based edit suggestions.

## Quick Start

### CLI

```bash
cd cli
cargo build --release

# First run prompts for mode selection
./target/release/grammar-check check document.txt

# Or pipe text directly
echo "He dont like the new changes." | ./target/release/grammar-check check
```

### Chrome Extension

```bash
cd extension
npm install
npm run build
```

Then load `extension/dist/` as an unpacked extension in `chrome://extensions/` (Developer mode).

## CLI Usage

### Check for errors

```bash
# Check a file (human-readable output)
grammar-check check document.txt

# Check from stdin
echo "She dont likes it." | grammar-check check

# JSON output for scripting
grammar-check check document.txt --format json
```

### Apply fixes

```bash
# Output corrected text to stdout
grammar-check fix document.txt

# Write to a new file
grammar-check fix document.txt --output fixed.txt

# Fix in place
grammar-check fix document.txt --in-place
```

### Per-invocation overrides

```bash
# Use a different model for one check
grammar-check check document.txt --model gpt-4o

# Point at an OpenAI-compatible API for one run
grammar-check check document.txt --base-url https://my-proxy.example.com/v1

# Both flags work on fix too
grammar-check fix document.txt --model gemini-2.0-flash --in-place
```

### Configuration

```bash
# Set operating mode
grammar-check config --mode local
grammar-check config --mode cloud --provider openai

# Store API key in OS keychain
grammar-check config --set-api-key

# Set model and custom base URL (persisted)
grammar-check config --set-model gpt-4o
grammar-check config --set-base-url https://my-proxy.example.com/v1

# Show current settings
grammar-check config --show
```

## Extension Features

- **Floating button** appears when hovering near any textarea
- **Inline highlights** — red for grammar errors, yellow for style issues (Shadow DOM isolated)
- **One-click fixes** — click a highlight to see the suggestion, click Apply to fix
- **Settings popup** — configure mode, provider, and API key
- **Keyboard shortcut** — `Ctrl+Shift+G` (Mac: `Cmd+Shift+G`) to check the focused textarea

## Project Structure

```
cli/                        # Rust CLI application
  src/
    commands/               # check, fix, config subcommands
    providers/              # Ollama + Cloud (OpenAI/Gemini) providers
    models/                 # Edit, Config types
    services/               # diff, chunker, validator, credentials
    output/                 # human-readable + JSON formatters

extension/                  # Chrome Extension (React + TypeScript)
  src/
    background/             # Manifest V3 service worker
    content/                # Content script, FloatingButton, HighlightOverlay, FixPopup
    popup/                  # Settings UI
    providers/              # Ollama + Cloud providers (mirrors CLI)
    store/                  # Zustand state management
    utils/                  # crypto, diffApplier, positionMapper, storage

shared/schemas/             # JSON Schema for LLM response format
specs/                      # Feature specifications and design docs
```

## Prerequisites

| Requirement | Version |
|-------------|---------|
| Rust | 1.75+ ([rustup.rs](https://rustup.rs)) |
| Node.js | 20+ (for extension) |
| Ollama | Latest (Local Mode only) |

For Local Mode, pull the model after installing Ollama:

```bash
ollama pull llama3
```

For Cloud Mode, get an API key from [OpenAI](https://platform.openai.com/api-keys) or [Google AI Studio](https://makersuite.google.com/app/apikey).

## How It Works

1. Text is captured from file/stdin (CLI) or textarea (extension)
2. Large documents are split into ~3,500-character semantic chunks with 400-char overlap
3. Each chunk is sent to the selected LLM provider with a system prompt requesting JSON edit suggestions
4. Responses are validated against the shared JSON Schema
5. Edit positions are merged, deduplicated, and presented as diffs or inline highlights

All positions use UTF-16 code unit indices for cross-platform consistency between Rust and JavaScript.

## Development

```bash
# CLI
cd cli
cargo test          # Run tests
cargo clippy        # Lint

# Extension
cd extension
npm run typecheck   # Type-check
npm run build       # Production build
npm run dev         # Watch mode
npm test            # Run tests
```

## Design Principles

- **Privacy First** — Local Mode makes zero network calls; API keys stored in OS keychain (CLI) or encrypted via Web Crypto API (extension)
- **Type Safety** — `#![deny(clippy::unwrap_used)]` in Rust; `strict: true` in TypeScript
- **Performance** — CLI startup <100ms; extension bundle <200KB
- **Minimal Dependencies** — Focused crates and lightweight frontend (React + Zustand, no heavy frameworks)

## License

MIT
