# Quickstart: Grammar Checker Tool

**Feature**: 001-grammar-checker
**Date**: 2026-01-06

This guide covers setup, building, and running the grammar checker tool in both CLI and Extension modes.

---

## Prerequisites

### For Local Mode (Ollama)
1. Install Ollama: https://ollama.ai/download
2. Pull the Llama 3 model:
   ```bash
   ollama pull llama3
   ```
3. Verify Ollama is running:
   ```bash
   curl http://localhost:11434
   ```

### For Cloud Mode
- **Gemini**: Get API key from https://makersuite.google.com/app/apikey
- **OpenAI**: Get API key from https://platform.openai.com/api-keys

### Development Tools
- **Rust**: Install via https://rustup.rs (1.75+)
- **Node.js**: LTS version (20.x+) from https://nodejs.org
- **pnpm**: `npm install -g pnpm` (recommended for extension)

---

## CLI Setup

### 1. Build the CLI

```bash
cd cli

# Debug build (faster compilation)
cargo build

# Release build (optimized)
cargo build --release
```

### 2. Configure Mode

On first run, the CLI will prompt for mode selection:

```bash
# Run any command to trigger setup
./target/release/grammar-check check --help

# Or configure directly
./target/release/grammar-check config --mode local
./target/release/grammar-check config --mode cloud --provider openai
```

### 3. Set Cloud API Key (if using Cloud Mode)

```bash
# The CLI will prompt for the API key and store it in OS keychain
./target/release/grammar-check config --set-api-key

# Or provide it directly (not recommended for security)
./target/release/grammar-check config --set-api-key "sk-..."
```

### 4. Basic Usage

```bash
# Check a file
./target/release/grammar-check check document.txt

# Check piped input
echo "He dont like the new changes." | ./target/release/grammar-check check

# Fix and output to stdout
./target/release/grammar-check fix document.txt

# Fix and write to file
./target/release/grammar-check fix document.txt --output fixed.txt

# Fix in place
./target/release/grammar-check fix document.txt --in-place

# JSON output for scripting
./target/release/grammar-check check document.txt --format json
```

---

## Extension Setup

### 1. Install Dependencies

```bash
cd extension
pnpm install
```

### 2. Development Mode

```bash
# Start dev server with hot reload
pnpm dev
```

### 3. Load in Chrome

1. Navigate to `chrome://extensions/`
2. Enable "Developer mode" (top right toggle)
3. Click "Load unpacked"
4. Select the `extension/dist` directory

### 4. Production Build

```bash
# Build for production
pnpm build

# The dist/ folder can be zipped for Chrome Web Store
```

### 5. Configure Extension

1. Click the extension icon in Chrome toolbar
2. Select mode: **Local** (Ollama) or **Cloud** (Gemini/OpenAI)
3. For Cloud mode, enter your API key

---

## Project Structure

```
grammar-checker/
├── cli/                    # Rust CLI application
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/       # check, fix subcommands
│   │   ├── providers/      # ollama, cloud providers
│   │   ├── models/         # Edit, Config types
│   │   ├── services/       # diff, chunker, validator
│   │   └── output/         # human, json formatters
│   └── tests/
│
├── extension/              # Chrome Extension
│   ├── package.json
│   ├── manifest.json
│   ├── src/
│   │   ├── background/     # Service worker
│   │   ├── content/        # Content scripts
│   │   ├── popup/          # Settings popup
│   │   ├── providers/      # API providers
│   │   └── store/          # Zustand state
│   └── tests/
│
├── shared/                 # Shared resources
│   └── schemas/
│       └── llm-response.schema.json
│
└── specs/                  # Feature specifications
    └── 001-grammar-checker/
        ├── spec.md
        ├── plan.md
        ├── research.md
        ├── data-model.md
        ├── quickstart.md   # This file
        └── contracts/
```

---

## Testing

### CLI Tests

```bash
cd cli

# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_edit_validation

# Run integration tests
cargo test --test integration
```

### Extension Tests

```bash
cd extension

# Run unit tests
pnpm test

# Run with coverage
pnpm test:coverage

# Run in watch mode
pnpm test:watch
```

---

## Common Commands

### CLI

| Command | Description |
|---------|-------------|
| `grammar-check check <file>` | Show suggested edits |
| `grammar-check fix <file>` | Output corrected text |
| `grammar-check fix <file> --in-place` | Modify file directly |
| `grammar-check config --mode <local\|cloud>` | Set operating mode |
| `grammar-check config --set-api-key` | Store cloud API key |
| `grammar-check --format json` | Machine-readable output |
| `grammar-check --help` | Show all options |

### Extension Development

| Command | Description |
|---------|-------------|
| `pnpm dev` | Start development server |
| `pnpm build` | Production build |
| `pnpm test` | Run tests |
| `pnpm lint` | Run ESLint |
| `pnpm typecheck` | TypeScript validation |

---

## Environment Variables

### CLI

| Variable | Description | Default |
|----------|-------------|---------|
| `GRAMMAR_CHECK_CONFIG` | Config file path | Platform-specific |
| `GRAMMAR_CHECK_OLLAMA_HOST` | Ollama API URL | `http://localhost:11434` |
| `GRAMMAR_CHECK_LOG_LEVEL` | Log verbosity | `warn` |

### Extension

The extension reads configuration from `chrome.storage.local` rather than environment variables.

---

## Troubleshooting

### CLI Issues

**"Ollama unavailable" error:**
```bash
# Check if Ollama is running
curl http://localhost:11434
# If not running, start it
ollama serve
```

**"API key not found" error:**
```bash
# Re-enter API key
./target/release/grammar-check config --set-api-key
```

**Slow startup (>100ms):**
```bash
# Use release build
cargo build --release
```

### Extension Issues

**Floating button not appearing:**
- Verify extension is enabled in `chrome://extensions/`
- Check console for errors (right-click → Inspect)
- Ensure the page has textarea elements

**Highlights not showing:**
- Check if mode is configured (click extension icon)
- For Local mode, verify Ollama is running
- Check browser console for API errors

**CORS errors with Ollama:**
- Ollama must allow localhost connections (default behavior)
- If using non-standard port, update extension settings

---

## Next Steps

1. **Run the CLI** with a sample document
2. **Load the extension** in Chrome developer mode
3. **Test both modes** (Local with Ollama, Cloud with API key)
4. **Check the spec.md** for acceptance criteria
5. **Review data-model.md** for type definitions
