/**
 * Configuration types for the browser extension.
 * Stored in chrome.storage.local (not sync, for privacy).
 */

export interface ExtensionConfig {
  mode: 'local' | 'cloud';
  cloudProvider?: 'gemini' | 'openai';
  /** Local Ollama host, defaults to http://localhost:11434 */
  localHost?: string;
  /** Encrypted API key (via Web Crypto API) */
  encryptedApiKey?: string;
  /** IV used for AES-GCM encryption */
  encryptionIv?: string;
}

export const DEFAULT_LOCAL_HOST = 'http://localhost:11434';

export const DEFAULT_CONFIG: ExtensionConfig = {
  mode: 'local',
  localHost: DEFAULT_LOCAL_HOST,
};
