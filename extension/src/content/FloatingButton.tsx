/**
 * FloatingButton component — appears near a textarea on hover.
 * Shows grammar check status and triggers the check action.
 */
import React, { useCallback } from 'react';
import { useGrammarStore } from '../store/useGrammarStore';

interface FloatingButtonProps {
  textareaId: string;
  getText: () => string;
  style?: React.CSSProperties;
}

export function FloatingButton({ textareaId, getText, style }: FloatingButtonProps) {
  const { isChecking, error, checkText, clearEdits } = useGrammarStore();

  const handleClick = useCallback(async () => {
    const text = getText();
    if (text.trim().length === 0) return;
    await checkText(text, textareaId);
  }, [getText, textareaId, checkText]);

  const handleClear = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      clearEdits();
    },
    [clearEdits]
  );

  const buttonStyle: React.CSSProperties = {
    position: 'absolute',
    zIndex: 2147483647,
    display: 'flex',
    alignItems: 'center',
    gap: '4px',
    padding: '4px 8px',
    borderRadius: '4px',
    border: 'none',
    cursor: 'pointer',
    fontSize: '12px',
    fontFamily: 'system-ui, sans-serif',
    fontWeight: 600,
    boxShadow: '0 2px 8px rgba(0,0,0,0.2)',
    transition: 'opacity 0.2s ease, transform 0.2s ease',
    backgroundColor: error ? '#ff4444' : '#4a90e2',
    color: '#fff',
    ...style,
  };

  if (isChecking) {
    return (
      <button style={buttonStyle} disabled aria-label="Checking grammar…">
        <span style={{ animation: 'spin 1s linear infinite', display: 'inline-block' }}>⟳</span>
        Checking…
      </button>
    );
  }

  if (error) {
    return (
      <button
        style={buttonStyle}
        onClick={handleClick}
        title={error}
        aria-label={`Grammar check failed: ${error}. Click to retry.`}
      >
        ✕ Error (retry)
      </button>
    );
  }

  return (
    <div style={{ display: 'flex', gap: '4px', position: 'absolute', ...style }}>
      <button
        style={{ ...buttonStyle, position: 'relative' }}
        onClick={handleClick}
        aria-label="Check grammar"
      >
        ✓ Check
      </button>
      <button
        style={{
          ...buttonStyle,
          position: 'relative',
          backgroundColor: '#888',
          padding: '4px 6px',
        }}
        onClick={handleClear}
        aria-label="Clear highlights"
        title="Clear highlights"
      >
        ✕
      </button>
    </div>
  );
}
