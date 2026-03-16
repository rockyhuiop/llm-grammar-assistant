/**
 * diffApplier: applies a single Edit to a textarea's value.
 */
import type { Edit } from '../types/edit';

/** Applies a single edit to a text string, returning the corrected string. */
export function applyEditToText(text: string, edit: Edit): string {
  return text.slice(0, edit.start_index) + edit.replacement + text.slice(edit.end_index);
}

/** Applies a single edit to a textarea element, updating its value. */
export function applyEditToTextarea(textarea: HTMLTextAreaElement, edit: Edit): void {
  const newValue = applyEditToText(textarea.value, edit);
  textarea.value = newValue;

  // Dispatch input event so frameworks (React, Vue) pick up the change
  textarea.dispatchEvent(new Event('input', { bubbles: true }));
  textarea.dispatchEvent(new Event('change', { bubbles: true }));
}

/**
 * Recalculates positions of remaining edits after one has been applied.
 * Edits after the applied edit shift by the replacement length difference.
 */
export function recalculatePositions(edits: Edit[], appliedEdit: Edit): Edit[] {
  const delta = appliedEdit.replacement.length - (appliedEdit.end_index - appliedEdit.start_index);

  return edits
    .filter((e) => e !== appliedEdit)
    .map((edit) => {
      if (edit.start_index >= appliedEdit.end_index) {
        return {
          ...edit,
          start_index: edit.start_index + delta,
          end_index: edit.end_index + delta,
        };
      }
      return edit;
    });
}
