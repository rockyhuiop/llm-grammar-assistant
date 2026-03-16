/**
 * GrammarHighlightOverlay — custom element using closed Shadow DOM.
 * Renders absolutely-positioned highlight spans over a textarea.
 *
 * Usage:
 *   const el = document.createElement('grammar-highlight-overlay');
 *   textarea.insertAdjacentElement('afterend', el);
 *   el.attachToTextarea(textarea);
 *   el.setEdits(edits);
 */
import type { Edit } from '../types/edit';
import { mapPositionToRect } from '../utils/positionMapper';

const GRAMMAR_COLOR = 'rgba(255, 107, 107, 0.4)';
const STYLE_COLOR = 'rgba(255, 224, 102, 0.4)';

const OVERLAY_CSS = `
:host {
  all: initial;
  position: absolute;
  contain: layout style paint;
  pointer-events: none;
  overflow: hidden;
}
.highlight {
  position: absolute;
  pointer-events: auto;
  cursor: pointer;
  border-radius: 2px;
  transition: opacity 0.15s;
}
.highlight:hover {
  opacity: 0.8 !important;
}
.highlight.grammar { background-color: ${GRAMMAR_COLOR}; }
.highlight.style   { background-color: ${STYLE_COLOR}; }
.tooltip {
  position: absolute;
  bottom: calc(100% + 4px);
  left: 0;
  background: #333;
  color: #fff;
  font: 12px/1.4 system-ui, sans-serif;
  padding: 4px 8px;
  border-radius: 4px;
  white-space: nowrap;
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.15s;
  z-index: 1;
}
.highlight:hover .tooltip { opacity: 1; }
`;

export class GrammarHighlightOverlay extends HTMLElement {
  private shadow: ShadowRoot;
  private textarea: HTMLTextAreaElement | null = null;
  private edits: Edit[] = [];
  private resizeObserver: ResizeObserver | null = null;

  constructor() {
    super();
    this.shadow = this.attachShadow({ mode: 'closed' });
    const style = document.createElement('style');
    style.textContent = OVERLAY_CSS;
    this.shadow.appendChild(style);
  }

  attachToTextarea(textarea: HTMLTextAreaElement): void {
    this.textarea = textarea;
    this.syncPosition();

    // Track textarea resize
    this.resizeObserver = new ResizeObserver(() => {
      this.syncPosition();
      this.render();
    });
    this.resizeObserver.observe(textarea);

    // Track scroll inside textarea
    textarea.addEventListener('scroll', () => this.render(), { passive: true });
  }

  setEdits(edits: Edit[]): void {
    this.edits = edits;
    this.render();
  }

  disconnectedCallback(): void {
    this.resizeObserver?.disconnect();
  }

  // ── Private ────────────────────────────────────────────────────────────────

  private syncPosition(): void {
    if (!this.textarea) return;
    const rect = this.textarea.getBoundingClientRect();
    const scrollX = window.scrollX;
    const scrollY = window.scrollY;

    this.style.cssText = `
      position: absolute;
      top: ${rect.top + scrollY}px;
      left: ${rect.left + scrollX}px;
      width: ${rect.width}px;
      height: ${rect.height}px;
      z-index: 2147483645;
      pointer-events: none;
    `;
  }

  private render(): void {
    if (!this.textarea) return;

    // Remove old highlights (keep the style element)
    const children = Array.from(this.shadow.children);
    for (const child of children) {
      if (child.tagName !== 'STYLE') child.remove();
    }

    for (const edit of this.edits) {
      const rects = mapPositionToRect(this.textarea, edit.start_index, edit.end_index);
      for (const rect of rects) {
        const span = document.createElement('div');
        span.className = `highlight ${edit.category}`;
        span.style.cssText = `
          top: ${rect.top}px;
          left: ${rect.left}px;
          width: ${rect.width}px;
          height: ${rect.height}px;
        `;

        // Tooltip
        const tooltip = document.createElement('div');
        tooltip.className = 'tooltip';
        const original = this.textarea.value.slice(edit.start_index, edit.end_index);
        tooltip.textContent = `${original} → ${edit.replacement}`;
        span.appendChild(tooltip);

        // Click → dispatch custom event for content script to handle
        span.addEventListener('click', (e) => {
          e.stopPropagation();
          this.dispatchEvent(
            new CustomEvent<Edit>('highlight-click', {
              detail: edit,
              bubbles: true,
              composed: true,
            })
          );
        });

        this.shadow.appendChild(span);
      }
    }
  }
}

if (!customElements.get('grammar-highlight-overlay')) {
  customElements.define('grammar-highlight-overlay', GrammarHighlightOverlay);
}
