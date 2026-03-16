/**
 * Web Crypto API utilities for API key encryption/decryption.
 * Uses AES-GCM with a device-derived key stored in chrome.storage.local.
 */

const CRYPTO_KEY_NAME = 'grammar_check_crypto_key';
const ALGORITHM = 'AES-GCM';

/** Gets or creates a persistent device-local encryption key. */
async function getOrCreateKey(): Promise<CryptoKey> {
  return new Promise((resolve, reject) => {
    chrome.storage.local.get(CRYPTO_KEY_NAME, async (result) => {
      const stored = result[CRYPTO_KEY_NAME] as number[] | undefined;
      if (stored) {
        try {
          const keyBytes = new Uint8Array(stored);
          const key = await crypto.subtle.importKey('raw', keyBytes, ALGORITHM, false, [
            'encrypt',
            'decrypt',
          ]);
          resolve(key);
          return;
        } catch {
          // Fall through to create new key
        }
      }

      // Generate new key
      const key = await crypto.subtle.generateKey({ name: ALGORITHM, length: 256 }, true, [
        'encrypt',
        'decrypt',
      ]);
      const exported = await crypto.subtle.exportKey('raw', key);
      const keyBytes = Array.from(new Uint8Array(exported));
      chrome.storage.local.set({ [CRYPTO_KEY_NAME]: keyBytes }, () => {
        if (chrome.runtime.lastError) {
          reject(new Error(chrome.runtime.lastError.message));
        } else {
          resolve(key);
        }
      });
    });
  });
}

/** Encrypts an API key using AES-GCM. Returns base64-encoded ciphertext and IV. */
export async function encryptApiKey(apiKey: string): Promise<{ encrypted: string; iv: string }> {
  const key = await getOrCreateKey();
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const encoded = new TextEncoder().encode(apiKey);
  const ciphertext = await crypto.subtle.encrypt({ name: ALGORITHM, iv }, key, encoded);

  return {
    encrypted: btoa(String.fromCharCode(...new Uint8Array(ciphertext))),
    iv: btoa(String.fromCharCode(...iv)),
  };
}

/** Decrypts a previously encrypted API key. */
export async function decryptApiKey(encryptedB64: string, ivB64: string): Promise<string> {
  const key = await getOrCreateKey();
  const ciphertext = Uint8Array.from(atob(encryptedB64), (c) => c.charCodeAt(0));
  const iv = Uint8Array.from(atob(ivB64), (c) => c.charCodeAt(0));

  const plaintext = await crypto.subtle.decrypt({ name: ALGORITHM, iv }, key, ciphertext);
  return new TextDecoder().decode(plaintext);
}
