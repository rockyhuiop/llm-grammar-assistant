/**
 * Extension popup — settings UI.
 * Allows the user to select Local/Cloud mode, cloud provider, and API key.
 */
import React, { useCallback, useEffect, useState } from 'react';
import { createRoot } from 'react-dom/client';
import type { ExtensionConfig } from '../types/config';
import { DEFAULT_CONFIG } from '../types/config';
import { loadConfig, saveConfig } from '../utils/storage';
import { encryptApiKey } from '../utils/crypto';

// ─── Styles (inline for isolation; popup is a separate HTML page) ─────────────

const S = {
  root: {
    fontFamily: 'system-ui, -apple-system, sans-serif',
    fontSize: '14px',
    color: '#1a1a1a',
    padding: '16px',
    width: '280px',
    boxSizing: 'border-box' as const,
  },
  heading: {
    margin: '0 0 16px',
    fontSize: '16px',
    fontWeight: 700,
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
  },
  section: { marginBottom: '16px' },
  label: {
    display: 'block',
    fontWeight: 600,
    marginBottom: '6px',
    fontSize: '12px',
    textTransform: 'uppercase' as const,
    color: '#555',
    letterSpacing: '0.05em',
  },
  radioGroup: { display: 'flex', gap: '12px' },
  radioLabel: { display: 'flex', alignItems: 'center', gap: '6px', cursor: 'pointer' },
  select: {
    width: '100%',
    padding: '6px 8px',
    border: '1px solid #ddd',
    borderRadius: '4px',
    fontSize: '13px',
    backgroundColor: '#fff',
  },
  inputRow: { display: 'flex', gap: '6px' },
  input: {
    flex: 1,
    padding: '6px 8px',
    border: '1px solid #ddd',
    borderRadius: '4px',
    fontSize: '13px',
  },
  toggleBtn: {
    padding: '6px 10px',
    border: '1px solid #ddd',
    borderRadius: '4px',
    background: '#f5f5f5',
    cursor: 'pointer',
    fontSize: '13px',
  },
  saveBtn: {
    width: '100%',
    padding: '8px',
    backgroundColor: '#4a90e2',
    color: '#fff',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
    fontWeight: 600,
    fontSize: '13px',
  },
  status: {
    marginTop: '10px',
    fontSize: '12px',
    textAlign: 'center' as const,
    minHeight: '16px',
  },
};

// ─── Component ────────────────────────────────────────────────────────────────

export function Popup() {
  const [config, setConfig] = useState<ExtensionConfig>(DEFAULT_CONFIG);
  const [apiKey, setApiKey] = useState('');
  const [showKey, setShowKey] = useState(false);
  const [status, setStatus] = useState('');
  const [saving, setSaving] = useState(false);

  // Load stored config on mount
  useEffect(() => {
    loadConfig().then(setConfig).catch(() => {});
  }, []);

  const handleSave = useCallback(async () => {
    setSaving(true);
    setStatus('');
    try {
      let updated = { ...config };

      if (config.mode === 'cloud' && apiKey.trim()) {
        const { encrypted, iv } = await encryptApiKey(apiKey.trim());
        updated = { ...updated, encryptedApiKey: encrypted, encryptionIv: iv };
      }

      await saveConfig(updated);
      setConfig(updated);
      setApiKey('');
      setStatus('✓ Settings saved');
    } catch (err) {
      setStatus(`Error: ${err instanceof Error ? err.message : 'Save failed'}`);
    } finally {
      setSaving(false);
    }
  }, [config, apiKey]);

  const hasStoredKey = Boolean(config.encryptedApiKey);

  return (
    <div style={S.root}>
      <h1 style={S.heading}>
        <span>✓</span>
        <span>Grammar Check</span>
      </h1>

      {/* Mode selector */}
      <div style={S.section}>
        <span style={S.label}>Mode</span>
        <div style={S.radioGroup}>
          {(['local', 'cloud'] as const).map((m) => (
            <label key={m} style={S.radioLabel}>
              <input
                type="radio"
                name="mode"
                value={m}
                checked={config.mode === m}
                onChange={() => setConfig((c) => ({ ...c, mode: m }))}
              />
              {m === 'local' ? 'Local (Ollama)' : 'Cloud API'}
            </label>
          ))}
        </div>
      </div>

      {/* Cloud-only settings */}
      {config.mode === 'cloud' && (
        <>
          {/* Provider selector */}
          <div style={S.section}>
            <label style={S.label} htmlFor="provider">
              Provider
            </label>
            <select
              id="provider"
              style={S.select}
              value={config.cloudProvider ?? 'gemini'}
              onChange={(e) =>
                setConfig((c) => ({
                  ...c,
                  cloudProvider: e.target.value as 'gemini' | 'openai',
                }))
              }
            >
              <option value="gemini">Gemini (Google)</option>
              <option value="openai">OpenAI (GPT-4o mini)</option>
            </select>
          </div>

          {/* API key input */}
          <div style={S.section}>
            <label style={S.label} htmlFor="apiKey">
              API Key{hasStoredKey ? ' (stored — enter to replace)' : ''}
            </label>
            <div style={S.inputRow}>
              <input
                id="apiKey"
                type={showKey ? 'text' : 'password'}
                placeholder={hasStoredKey ? '••••••••' : 'Paste API key…'}
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                style={S.input}
                autoComplete="off"
                spellCheck={false}
              />
              <button
                style={S.toggleBtn}
                onClick={() => setShowKey((v) => !v)}
                aria-label={showKey ? 'Hide API key' : 'Show API key'}
              >
                {showKey ? '🙈' : '👁'}
              </button>
            </div>
          </div>
        </>
      )}

      {/* Local-only settings */}
      {config.mode === 'local' && (
        <div style={S.section}>
          <label style={S.label} htmlFor="ollamaHost">
            Ollama Host
          </label>
          <input
            id="ollamaHost"
            type="text"
            value={config.localHost ?? 'http://localhost:11434'}
            onChange={(e) => setConfig((c) => ({ ...c, localHost: e.target.value }))}
            style={{ ...S.input, width: '100%', boxSizing: 'border-box' }}
          />
        </div>
      )}

      <button style={S.saveBtn} onClick={handleSave} disabled={saving}>
        {saving ? 'Saving…' : 'Save Settings'}
      </button>

      <div
        style={{
          ...S.status,
          color: status.startsWith('✓') ? '#38a169' : status ? '#e53e3e' : 'transparent',
        }}
      >
        {status || '.'}
      </div>
    </div>
  );
}

// ─── Mount ────────────────────────────────────────────────────────────────────

const root = document.getElementById('root');
if (root) {
  createRoot(root).render(
    <React.StrictMode>
      <Popup />
    </React.StrictMode>
  );
}
