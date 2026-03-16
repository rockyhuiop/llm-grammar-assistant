/**
 * chrome.storage.local wrapper for extension configuration.
 */
import { type ExtensionConfig, DEFAULT_CONFIG } from '../types/config';

const STORAGE_KEY = 'grammar_check_config';

/** Loads config from chrome.storage.local, returning defaults if none stored. */
export async function loadConfig(): Promise<ExtensionConfig> {
  return new Promise((resolve) => {
    chrome.storage.local.get(STORAGE_KEY, (result) => {
      const stored = result[STORAGE_KEY] as ExtensionConfig | undefined;
      resolve(stored ?? DEFAULT_CONFIG);
    });
  });
}

/** Persists config to chrome.storage.local. */
export async function saveConfig(config: ExtensionConfig): Promise<void> {
  return new Promise((resolve, reject) => {
    chrome.storage.local.set({ [STORAGE_KEY]: config }, () => {
      if (chrome.runtime.lastError) {
        reject(new Error(chrome.runtime.lastError.message));
      } else {
        resolve();
      }
    });
  });
}

/** Clears stored config and credentials (e.g. on uninstall). */
export async function clearConfig(): Promise<void> {
  return new Promise((resolve) => {
    chrome.storage.local.remove(STORAGE_KEY, () => resolve());
  });
}
