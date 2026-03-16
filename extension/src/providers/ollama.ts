/**
 * OllamaProvider for the browser extension.
 * Calls local Ollama instance (localhost:11434) — no external network calls.
 */
import type { Edit, LlmResponse } from '../types/edit';
import type { Provider } from './index';
import { GRAMMAR_CHECK_SYSTEM_PROMPT, formatUserMessage } from './prompts';

export class OllamaProvider implements Provider {
  readonly providerName = 'ollama';
  readonly modelName: string;
  private readonly host: string;

  constructor(host = 'http://localhost:11434', model = 'llama3') {
    this.host = host;
    this.modelName = model;
  }

  async check(text: string): Promise<Edit[]> {
    const response = await fetch(`${this.host}/api/chat`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        model: this.modelName,
        messages: [
          { role: 'system', content: GRAMMAR_CHECK_SYSTEM_PROMPT },
          { role: 'user', content: formatUserMessage(text) },
        ],
        stream: false,
        options: { temperature: 0.1 },
      }),
    });

    if (!response.ok) {
      throw new Error(`Ollama returned HTTP ${response.status}. Is it running at ${this.host}?`);
    }

    const data = (await response.json()) as { message?: { content?: string } };
    const content = data.message?.content;
    if (!content) {
      throw new Error('Unexpected Ollama response shape');
    }

    return parseAndValidate(content, text);
  }
}

function parseAndValidate(json: string, originalText: string): Edit[] {
  let parsed: LlmResponse;
  try {
    parsed = JSON.parse(json) as LlmResponse;
  } catch {
    throw new Error(`Invalid JSON from LLM: ${json.slice(0, 200)}`);
  }

  const textLen = [...originalText].length; // JS string length (UTF-16)
  return parsed.edits.filter((edit) => {
    if (edit.start_index >= edit.end_index) return false;
    if (edit.end_index > textLen) return false;
    if (originalText.slice(edit.start_index, edit.end_index) === edit.replacement) return false;
    return true;
  });
}
