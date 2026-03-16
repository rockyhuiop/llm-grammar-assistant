# Feature Specification: Grammar Checker Tool

**Feature Branch**: `001-grammar-checker`
**Created**: 2026-01-06
**Status**: Draft
**Input**: User description: "Build a grammar checking tool with two interfaces: a Rust CLI and a Chrome Extension. The tool operates in two modes: Local (via Ollama/Llama 3) and Cloud (via Gemini/OpenAI APIs). It relies on a 'Diff Pipeline': The Client sends text + JSON Schema to the LLM, and the LLM returns a list of edits (start_index, end_index, replacement). CLI: Handles text via stdin or file arguments. Supports check (view diffs) and fix (apply diffs). Extension: Injects a floating button into textarea elements. On click, it overlays highlights (Red for grammar, Yellow for style) using Shadow DOM to prevent CSS conflicts."

## Clarifications

### Session 2026-01-06

- Q: What is the default operating mode on first run? → A: No default; require explicit mode selection before first use (blocking prompt)
- Q: What output formats should the CLI support? → A: Support both human-readable (default) and JSON via `--format` flag
- Q: What is the maximum document size? → A: 50,000 characters recommended; chunk larger documents automatically for processing
- Q: How should Cloud API errors be handled? → A: Retry transient errors (3 attempts, exponential backoff); display error and exit after exhaustion
- Q: How should the extension activate on websites? → A: Always active on all sites; floating button hidden until user hovers near textarea

## User Scenarios & Testing *(mandatory)*

### User Story 1 - CLI Grammar Check (Priority: P1)

A writer wants to quickly check a document for grammar errors from the command line without modifying the original file. They pipe their text or provide a file path, and the tool displays suggested corrections with context so they can review what would change.

**Why this priority**: This is the foundational capability. The diff pipeline must work correctly before any interface can be useful. CLI provides the fastest feedback loop for validating core logic.

**Independent Test**: Can be fully tested by running the CLI with sample text containing known grammar errors and verifying the output shows correct edit suggestions with position information.

**Acceptance Scenarios**:

1. **Given** a text file with grammar errors, **When** user runs `grammar-check check input.txt`, **Then** the tool displays a list of suggested edits showing original text, replacement, and position
2. **Given** text piped via stdin, **When** user runs `echo "He dont like it" | grammar-check check`, **Then** the tool outputs the suggested correction ("dont" → "doesn't") with line/position context
3. **Given** a document with no errors, **When** user runs check command, **Then** the tool reports "No issues found" and exits with code 0
4. **Given** Local Mode is configured, **When** user runs check command offline, **Then** the tool processes text without any network requests

---

### User Story 2 - CLI Grammar Fix (Priority: P2)

A writer wants to automatically apply all suggested grammar corrections to their document. They run a single command and the tool outputs the corrected text (or writes to a file), allowing them to quickly fix an entire document.

**Why this priority**: Extends the check capability with automated fixing. Depends on P1 working correctly but adds significant value by automating the correction workflow.

**Independent Test**: Can be tested by running the fix command on a file with known errors and comparing output against expected corrected text.

**Acceptance Scenarios**:

1. **Given** a text file with grammar errors, **When** user runs `grammar-check fix input.txt`, **Then** the corrected text is written to stdout
2. **Given** the `--output` flag is provided, **When** user runs `grammar-check fix input.txt --output fixed.txt`, **Then** the corrected text is written to the specified file
3. **Given** the `--in-place` flag is provided, **When** user runs `grammar-check fix input.txt --in-place`, **Then** the original file is modified with corrections
4. **Given** overlapping edit suggestions exist, **When** user runs fix command, **Then** edits are applied in a consistent order that produces valid output

---

### User Story 3 - Browser Extension Highlighting (Priority: P3)

A user writing in a web application (email, social media, document editor) wants to see grammar issues highlighted directly in their text field. They click a floating button, and the extension overlays color-coded highlights on problematic text without disrupting the page's styling.

**Why this priority**: This delivers the visual browser experience but requires the diff pipeline to work first. More complex due to DOM manipulation and CSS isolation requirements.

**Independent Test**: Can be tested by typing text with grammar errors in any textarea, clicking the check button, and verifying highlights appear with correct colors (red for grammar, yellow for style).

**Acceptance Scenarios**:

1. **Given** a webpage with a textarea element, **When** the user hovers near the textarea, **Then** a floating grammar-check button fades in near the textarea
2. **Given** the user has typed text with grammar errors, **When** user clicks the floating button, **Then** grammar errors are highlighted in red and style issues in yellow
3. **Given** highlights are displayed, **When** user hovers over a highlight, **Then** a tooltip shows the suggested replacement
4. **Given** the page has complex CSS styling, **When** highlights are rendered, **Then** highlights display correctly without being affected by page styles (Shadow DOM isolation)

---

### User Story 4 - Browser Extension Quick Fix (Priority: P4)

A user viewing highlighted errors wants to quickly apply a suggested fix. They click on a highlighted word, see the suggestion, and can accept it with one click to update the text field content.

**Why this priority**: Builds on P3 highlighting to complete the browser experience. Requires both highlighting and edit application to work.

**Independent Test**: Can be tested by clicking a highlighted error and verifying the text field content updates with the correction.

**Acceptance Scenarios**:

1. **Given** a highlighted grammar error, **When** user clicks the highlight, **Then** a popup shows the original text and suggested replacement
2. **Given** the fix popup is displayed, **When** user clicks "Apply", **Then** the textarea content is updated with the correction and the highlight is removed
3. **Given** multiple errors exist, **When** user fixes one error, **Then** remaining highlights adjust their positions to account for text length changes

---

### User Story 5 - Mode Selection and Configuration (Priority: P5)

A user wants to choose between Local Mode (offline, privacy-focused) and Cloud Mode (higher quality, requires API key). They configure their preferred mode and credentials, and the tool remembers this setting across sessions.

**Why this priority**: Configuration is needed for real-world usage but core functionality should work with sensible defaults first.

**Independent Test**: Can be tested by configuring each mode and verifying the tool uses the correct backend for processing.

**Acceptance Scenarios**:

1. **Given** no configuration exists, **When** user runs the tool for the first time, **Then** the tool displays a blocking prompt requiring explicit mode selection (Local or Cloud) before proceeding
2. **Given** Cloud Mode is selected, **When** user provides API credentials, **Then** credentials are stored securely (OS keychain) and not in plain text files
3. **Given** Local Mode is selected, **When** user runs the tool, **Then** processing happens entirely on-device with no network calls
4. **Given** the configured cloud service is unavailable, **When** user runs the tool, **Then** the system retries up to 3 times with exponential backoff, then displays a descriptive error message and exits

---

### Edge Cases

- What happens when the input text is empty?
  - Tool should return immediately with "No text to check" message
- What happens when the LLM returns malformed edit positions (out of bounds)?
  - Tool should validate edits and skip/warn about invalid positions
- What happens when the user's text contains special characters or unicode?
  - Edit positions must correctly handle multi-byte characters
- How does the extension handle dynamically created textareas (SPA apps)?
  - Extension should observe DOM mutations and inject buttons into new textareas
- What happens when multiple textareas exist on a page?
  - Each textarea should have its own floating button and independent state
- What happens when the text changes while analysis is in progress?
  - CLI: Not applicable (input is read once)
  - Extension: MUST cancel pending analysis and clear stale highlights; user can re-trigger check via floating button

## Requirements *(mandatory)*

### Functional Requirements

**Core Diff Pipeline**
- **FR-001**: System MUST accept text input and return a list of edit suggestions
- **FR-002**: Each edit suggestion MUST include: start position, end position, replacement text, and issue category (grammar or style)
- **FR-003**: System MUST validate that edit positions are within text bounds before returning results
- **FR-004**: System MUST handle text with unicode characters, correctly calculating positions based on character indices (not byte indices)
- **FR-005**: System MUST process documents up to 50,000 characters without chunking
- **FR-006**: For documents exceeding 50,000 characters, system MUST automatically chunk the text and process segments sequentially, merging results with correctly adjusted positions

**CLI Interface**
- **FR-007**: CLI MUST accept text via stdin (piped input)
- **FR-008**: CLI MUST accept text via file path argument
- **FR-009**: CLI MUST provide a `check` subcommand that displays edits without modifying input
- **FR-010**: CLI MUST provide a `fix` subcommand that outputs corrected text
- **FR-011**: CLI MUST support `--output <path>` flag to write corrected text to a file
- **FR-012**: CLI MUST support `--in-place` flag to modify the input file directly
- **FR-013**: CLI MUST display edit context showing surrounding text for each suggestion
- **FR-014**: CLI MUST exit with code 0 on success, non-zero on errors
- **FR-015**: CLI MUST support `--format` flag with options: `human` (default, colored terminal output) and `json` (machine-parseable structured output)

**Browser Extension**
- **FR-016**: Extension MUST be active on all websites by default (no per-site activation required)
- **FR-017**: Extension MUST keep the floating button hidden until user hovers near a textarea element, then fade it in
- **FR-018**: Extension MUST use Shadow DOM to isolate highlight styles from page CSS
- **FR-019**: Extension MUST highlight grammar issues in red (#FF6B6B or similar visible red)
- **FR-020**: Extension MUST highlight style issues in yellow (#FFE066 or similar visible yellow)
- **FR-021**: Extension MUST display replacement suggestions when user interacts with highlights
- **FR-022**: Extension MUST update textarea content when user accepts a fix
- **FR-023**: Extension MUST handle dynamically created textareas in single-page applications

**Operating Modes**
- **FR-024**: System MUST support Local Mode using an on-device language model
- **FR-025**: System MUST support Cloud Mode using remote language model APIs
- **FR-026**: In Local Mode, system MUST NOT make any network requests
- **FR-027**: In Cloud Mode, system MUST store API credentials in the operating system's secure credential storage (not plain text)
- **FR-028**: System MUST allow users to configure their preferred operating mode
- **FR-029**: System MUST persist user configuration across sessions
- **FR-030**: On first run with no existing configuration, system MUST display a blocking prompt requiring explicit mode selection before any grammar checking can proceed
- **FR-031**: In Cloud Mode, system MUST retry transient API errors up to 3 times with exponential backoff before failing
- **FR-032**: After retry exhaustion, system MUST display a descriptive error message and exit with non-zero status code

### Key Entities

- **Edit**: Represents a single suggested correction. Contains start position (character index), end position (character index), replacement text, category (grammar/style), and optional explanation
- **CheckResult**: Collection of edits for a given text input. Contains original text, list of edits, and processing metadata (mode used, processing time)
- **Configuration**: User preferences and credentials. Contains operating mode (local/cloud), cloud provider selection, and credential reference (keychain identifier)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can check a 1000-word document and see results within 10 seconds in Cloud Mode, 30 seconds in Local Mode
- **SC-002**: CLI tool is ready to accept input within 100 milliseconds of invocation
- **SC-003**: 95% of suggested grammar corrections are accurate (true positives) as measured by user acceptance rate
- **SC-004**: Extension highlights appear within 2 seconds of clicking the check button for typical paragraph-length text
- **SC-005**: Zero user data is transmitted when operating in Local Mode (verifiable via network inspection)
- **SC-006**: Users can complete the check-review-fix workflow for a document in under 5 minutes
- **SC-007**: Extension functions correctly on the top 10 webmail and document editing sites without CSS conflicts
- **SC-008**: 90% of users successfully complete their first grammar check without consulting documentation

## Assumptions

- Users have Ollama installed locally if they wish to use Local Mode
- Users can obtain API keys for Gemini or OpenAI if they wish to use Cloud Mode
- Target browsers support Manifest V3 extension APIs and Shadow DOM
- Textarea elements are the primary text input mechanism (contenteditable divs are out of scope for initial release)
- English language support only for initial release; additional languages may be added later
