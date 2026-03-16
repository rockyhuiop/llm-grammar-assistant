/**
 * Zustand store for grammar checking state.
 * Manages edits, checking status, and per-textarea state.
 */
import { create } from 'zustand';
import type { Edit } from '../types/edit';
import { createProvider } from '../providers/index';
import { recalculatePositions } from '../utils/diffApplier';

interface GrammarStore {
  // State
  isChecking: boolean;
  currentTextareaId: string | null;
  edits: Edit[];
  error: string | null;

  // Actions
  checkText: (text: string, textareaId: string) => Promise<void>;
  applyEdit: (edit: Edit) => void;
  clearEdits: () => void;
  setError: (error: string | null) => void;
}

export const useGrammarStore = create<GrammarStore>((set, get) => ({
  isChecking: false,
  currentTextareaId: null,
  edits: [],
  error: null,

  checkText: async (text: string, textareaId: string) => {
    if (text.trim().length === 0) {
      set({ edits: [], currentTextareaId: textareaId, error: null });
      return;
    }

    set({ isChecking: true, currentTextareaId: textareaId, error: null });

    try {
      const provider = await createProvider();
      const edits = await provider.check(text);
      set({ edits, isChecking: false });
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Grammar check failed';
      set({ isChecking: false, edits: [], error: message });
    }
  },

  applyEdit: (edit: Edit) => {
    const { edits } = get();
    const remaining = recalculatePositions(edits, edit);
    set({ edits: remaining });
  },

  clearEdits: () => {
    set({ edits: [], currentTextareaId: null, error: null });
  },

  setError: (error: string | null) => {
    set({ error });
  },
}));
