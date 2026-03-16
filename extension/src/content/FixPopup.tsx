/**
 * FixPopup component — shown when clicking a highlighted error.
 * Displays the original text and suggested replacement with an Apply button.
 */
import React, { useCallback } from 'react';
import type { Edit } from '../types/edit';
import { useGrammarStore } from '../store/useGrammarStore';
import { applyEditToTextarea } from '../utils/diffApplier';

interface FixPopupProps {
  edit: Edit;
  originalText: string;
  textarea: HTMLTextAreaElement;
  onClose: () => void;
  style?: React.CSSProperties;
}

export function FixPopup({ edit, originalText, textarea, onClose, style }: FixPopupProps) {
  const { applyEdit } = useGrammarStore();

  const original = originalText.slice(edit.start_index, edit.end_index);

  const handleApply = useCallback(() => {
    applyEditToTextarea(textarea, edit);
    applyEdit(edit);
    onClose();
  }, [textarea, edit, applyEdit, onClose]);

  const popupStyle: React.CSSProperties = {
    position: 'absolute',
    zIndex: 2147483647,
    backgroundColor: '#fff',
    border: '1px solid #ddd',
    borderRadius: '6px',
    padding: '10px 12px',
    boxShadow: '0 4px 16px rgba(0,0,0,0.15)',
    fontFamily: 'system-ui, sans-serif',
    fontSize: '13px',
    minWidth: '200px',
    maxWidth: '320px',
    ...style,
  };

  const categoryColor = edit.category === 'grammar' ? '#e53e3e' : '#d69e2e';

  return (
    <div style={popupStyle} role="dialog" aria-label="Grammar fix suggestion">
      <div style={{ marginBottom: '8px' }}>
        <span
          style={{
            fontSize: '11px',
            fontWeight: 700,
            textTransform: 'uppercase',
            color: categoryColor,
            letterSpacing: '0.05em',
          }}
        >
          {edit.category}
        </span>
      </div>

      <div style={{ marginBottom: '8px', display: 'flex', alignItems: 'center', gap: '8px' }}>
        <span
          style={{
            textDecoration: 'line-through',
            color: '#e53e3e',
            fontWeight: 500,
          }}
        >
          {original}
        </span>
        <span style={{ color: '#888' }}>→</span>
        <span style={{ color: '#38a169', fontWeight: 600 }}>{edit.replacement}</span>
      </div>

      {edit.explanation && (
        <div style={{ marginBottom: '10px', color: '#666', fontSize: '12px', lineHeight: 1.4 }}>
          {edit.explanation}
        </div>
      )}

      <div style={{ display: 'flex', gap: '6px' }}>
        <button
          onClick={handleApply}
          style={{
            flex: 1,
            padding: '5px 10px',
            backgroundColor: '#4a90e2',
            color: '#fff',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
            fontWeight: 600,
            fontSize: '12px',
          }}
        >
          Apply
        </button>
        <button
          onClick={onClose}
          style={{
            padding: '5px 10px',
            backgroundColor: '#f5f5f5',
            color: '#333',
            border: '1px solid #ddd',
            borderRadius: '4px',
            cursor: 'pointer',
            fontSize: '12px',
          }}
        >
          Dismiss
        </button>
      </div>
    </div>
  );
}
