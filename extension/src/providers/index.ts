/**
 * Provider interface and factory for the browser extension.
 * Both OllamaProvider and CloudProvider implement this contract.
 */
import type { Edit } from '../types/edit';
import type { ExtensionConfig } from '../types/config';
import { OllamaProvider } from './ollama';
import { CloudProvider } from './cloud';
import { loadConfig } from '../utils/storage';

export interface Provider {
  check(text: string): Promise<Edit[]>;
  readonly providerName: string;
  readonly modelName: string;
}

/** Creates the appropriate provider based on stored config. */
export async function createProvider(): Promise<Provider> {
  const config: ExtensionConfig = await loadConfig();

  if (config.mode === 'local') {
    return new OllamaProvider(config.localHost);
  }

  // Cloud mode: decrypt API key
  const { decryptApiKey } = await import('../utils/crypto');
  const apiKey = config.encryptedApiKey && config.encryptionIv
    ? await decryptApiKey(config.encryptedApiKey, config.encryptionIv)
    : '';

  if (!apiKey) {
    throw new Error('No API key configured. Open extension settings to add one.');
  }

  return new CloudProvider(config.cloudProvider ?? 'gemini', apiKey);
}

