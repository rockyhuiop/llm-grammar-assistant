/**
 * Edit and CheckResult types for the browser extension.
 * Matches the shared JSON Schema contract (llm-response.schema.json).
 * Positions are UTF-16 code unit indices (same as JavaScript string indices).
 */

export interface Edit {
  /** Start position (UTF-16 code unit index, 0-based, inclusive) */
  start_index: number;
  /** End position (exclusive, UTF-16 code unit index) */
  end_index: number;
  /** Suggested replacement text. Empty string for deletions. */
  replacement: string;
  /** Issue type: grammar error (red) or style suggestion (yellow) */
  category: 'grammar' | 'style';
  /** Optional explanation of why this correction is suggested */
  explanation?: string;
}

/** The wrapper object returned by the LLM */
export interface LlmResponse {
  edits: Edit[];
}

/** Result of a complete grammar check operation */
export interface CheckResult {
  original_text: string;
  edits: Edit[];
  provider: string;
  processing_time_ms: number;
}
