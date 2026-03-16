/**
 * CloudProvider for the browser extension.
 * Supports OpenAI and Gemini APIs with retry logic.
 */
import type { Edit, LlmResponse } from '../types/edit';
import type { Provider } from './index';
import { GRAMMAR_CHECK_SYSTEM_PROMPT, formatUserMessage } from './prompts';

const JSON_SCHEMA = {
  type: 'object',
  required: ['edits'],
  properties: {
    edits: {
      type: 'array',
      items: {
        type: 'object',
        required: ['start_index', 'end_index', 'replacement', 'category'],
        properties: {
          start_index: { type: 'integer', minimum: 0 },
          end_index: { type: 'integer', minimum: 1 },
          replacement: { type: 'string' },
          category: { type: 'string', enum: ['grammar', 'style'] },
          explanation: { type: 'string' },
        },
        additionalProperties: false,
      },
    },
  },
  additionalProperties: false,
};

export class CloudProvider implements Provider {
  readonly providerName: string;
  readonly modelName: string;
  private readonly apiKey: string;
  private readonly type: 'openai' | 'gemini';

  constructor(type: 'openai' | 'gemini', apiKey: string, model?: string) {
    this.type = type;
    this.apiKey = apiKey;
    this.providerName = type;
    this.modelName = model ?? (type === 'openai' ? 'gpt-4o-mini' : 'gemini-2.0-flash');
  }

  async check(text: string): Promise<Edit[]> {
    return this.withRetry(() =>
      this.type === 'openai' ? this.callOpenAI(text) : this.callGemini(text)
    );
  }

  private async withRetry<T>(fn: () => Promise<T>): Promise<T> {
    const maxAttempts = 3;
    let lastError: unknown;
    for (let attempt = 0; attempt < maxAttempts; attempt++) {
      if (attempt > 0) {
        await new Promise((r) => setTimeout(r, (1 << (attempt - 1)) * 1000 + 50));
      }
      try {
        return await fn();
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        const retryable = msg.includes('429') || msg.includes('502') || msg.includes('503');
        if (!retryable) throw err;
        lastError = err;
      }
    }
    throw lastError ?? new Error('All retry attempts failed');
  }

  private async callOpenAI(text: string): Promise<Edit[]> {
    const response = await fetch('https://api.openai.com/v1/chat/completions', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${this.apiKey}`,
      },
      body: JSON.stringify({
        model: this.modelName,
        messages: [
          { role: 'system', content: GRAMMAR_CHECK_SYSTEM_PROMPT },
          { role: 'user', content: formatUserMessage(text) },
        ],
        response_format: {
          type: 'json_schema',
          json_schema: { name: 'grammar_check', strict: true, schema: JSON_SCHEMA },
        },
      }),
    });

    if (response.status === 401) {
      throw new Error('OpenAI authentication failed. Check your API key in extension settings.');
    }
    if (!response.ok) {
      throw new Error(`OpenAI returned HTTP ${response.status}`);
    }

    const data = (await response.json()) as { choices?: Array<{ message?: { content?: string } }> };
    const content = data.choices?.[0]?.message?.content;
    if (!content) throw new Error('Unexpected OpenAI response shape');

    return parseEdits(content, text);
  }

  private async callGemini(text: string): Promise<Edit[]> {
    const url = `https://generativelanguage.googleapis.com/v1beta/models/${this.modelName}/generateContent?key=${this.apiKey}`;
    const response = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        contents: [
          {
            parts: [
              { text: GRAMMAR_CHECK_SYSTEM_PROMPT },
              { text: formatUserMessage(text) },
            ],
          },
        ],
        generationConfig: {
          responseMimeType: 'application/json',
          responseSchema: {
            type: 'OBJECT',
            required: ['edits'],
            properties: {
              edits: {
                type: 'ARRAY',
                items: {
                  type: 'OBJECT',
                  required: ['start_index', 'end_index', 'replacement', 'category'],
                  properties: {
                    start_index: { type: 'INTEGER' },
                    end_index: { type: 'INTEGER' },
                    replacement: { type: 'STRING' },
                    category: { type: 'STRING', enum: ['grammar', 'style'] },
                    explanation: { type: 'STRING' },
                  },
                },
              },
            },
          },
          temperature: 0.1,
        },
      }),
    });

    if (!response.ok) {
      throw new Error(`Gemini returned HTTP ${response.status}`);
    }

    const data = (await response.json()) as {
      candidates?: Array<{ content?: { parts?: Array<{ text?: string }> } }>;
    };
    const content = data.candidates?.[0]?.content?.parts?.[0]?.text;
    if (!content) throw new Error('Unexpected Gemini response shape');

    return parseEdits(content, text);
  }
}

function parseEdits(json: string, originalText: string): Edit[] {
  let parsed: LlmResponse;
  try {
    parsed = JSON.parse(json) as LlmResponse;
  } catch {
    throw new Error(`Invalid JSON from LLM: ${json.slice(0, 200)}`);
  }

  const textLen = originalText.length;
  return parsed.edits.filter((edit) => {
    if (edit.start_index >= edit.end_index) return false;
    if (edit.end_index > textLen) return false;
    if (originalText.slice(edit.start_index, edit.end_index) === edit.replacement) return false;
    return true;
  });
}
