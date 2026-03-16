/**
 * Content script entry point.
 * Detects textareas, injects a floating button on hover, and manages
 * highlight overlays and fix popups via React roots.
 *
 * Architecture: one React root per UI layer, mounted into isolated
 * container divs appended directly to the page body.
 */
import React from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { FloatingButton } from './FloatingButton';
import { FixPopup } from './FixPopup';
import { GrammarHighlightOverlay } from './HighlightOverlay';
import { useGrammarStore } from '../store/useGrammarStore';
import type { Edit } from '../types/edit';

// ─── Types ─────────────────────────────────────────────────────────────────

interface TextareaEntry {
  textarea: HTMLTextAreaElement;
  id: string;
  overlay: GrammarHighlightOverlay;
  buttonContainer: HTMLDivElement;
  buttonRoot: Root;
  cleanup: () => void;
}

// ─── State ──────────────────────────────────────────────────────────────────

const registered = new Map<HTMLTextAreaElement, TextareaEntry>();
let nextId = 0;
let popupContainer: HTMLDivElement | null = null;
let popupRoot: Root | null = null;

// ─── Textarea registration ───────────────────────────────────────────────────

function registerTextarea(textarea: HTMLTextAreaElement): void {
  if (registered.has(textarea)) return;

  const id = `gc-textarea-${++nextId}`;

  // --- Highlight overlay (custom element / Shadow DOM) ---
  const overlay = document.createElement('grammar-highlight-overlay') as GrammarHighlightOverlay;
  textarea.insertAdjacentElement('afterend', overlay);
  overlay.attachToTextarea(textarea);

  // --- Floating button container ---
  const buttonContainer = document.createElement('div');
  buttonContainer.style.cssText =
    'position:absolute;z-index:2147483646;pointer-events:auto;display:none;';
  document.body.appendChild(buttonContainer);

  const buttonRoot = createRoot(buttonContainer);

  const entry: TextareaEntry = {
    textarea,
    id,
    overlay,
    buttonContainer,
    buttonRoot,
    cleanup: () => {},
  };

  // --- Hover show/hide ---
  const HOVER_MARGIN = 32; // px from textarea edge to still show button

  function positionButton(): void {
    const rect = textarea.getBoundingClientRect();
    const scrollX = window.scrollX;
    const scrollY = window.scrollY;
    buttonContainer.style.top = `${rect.bottom + scrollY + 4}px`;
    buttonContainer.style.left = `${rect.right + scrollX - 120}px`;
  }

  function showButton(): void {
    positionButton();
    buttonContainer.style.display = 'block';
    buttonRoot.render(
      <React.StrictMode>
        <FloatingButton
          textareaId={id}
          getText={() => textarea.value}
        />
      </React.StrictMode>
    );
  }

  function hideButton(): void {
    buttonContainer.style.display = 'none';
  }

  function isNearTextarea(e: MouseEvent): boolean {
    const rect = textarea.getBoundingClientRect();
    return (
      e.clientX >= rect.left - HOVER_MARGIN &&
      e.clientX <= rect.right + HOVER_MARGIN &&
      e.clientY >= rect.top - HOVER_MARGIN &&
      e.clientY <= rect.bottom + HOVER_MARGIN
    );
  }

  const onMouseMove = (e: MouseEvent): void => {
    if (isNearTextarea(e)) {
      showButton();
    } else {
      // Only hide if pointer is not over the button itself
      const bRect = buttonContainer.getBoundingClientRect();
      const overButton =
        e.clientX >= bRect.left &&
        e.clientX <= bRect.right &&
        e.clientY >= bRect.top &&
        e.clientY <= bRect.bottom;
      if (!overButton) hideButton();
    }
  };

  document.addEventListener('mousemove', onMouseMove);

  // Re-position button on scroll / resize
  const onScroll = (): void => positionButton();
  window.addEventListener('scroll', onScroll, { passive: true });

  // --- Subscribe to store: update overlay when edits change ---
  const unsubscribe = useGrammarStore.subscribe((state) => {
    if (state.currentTextareaId === id) {
      overlay.setEdits(state.edits);
    }
  });

  // --- Handle highlight click → show fix popup ---
  overlay.addEventListener('highlight-click', ((e: CustomEvent<Edit>) => {
    showFixPopup(e.detail, textarea);
  }) as EventListener);

  // --- Text change detection: clear stale highlights ---
  const onInput = (): void => {
    const store = useGrammarStore.getState();
    if (store.currentTextareaId === id && store.edits.length > 0) {
      store.clearEdits();
      overlay.setEdits([]);
    }
  };
  textarea.addEventListener('input', onInput);

  entry.cleanup = () => {
    document.removeEventListener('mousemove', onMouseMove);
    window.removeEventListener('scroll', onScroll);
    textarea.removeEventListener('input', onInput);
    unsubscribe();
    buttonRoot.unmount();
    buttonContainer.remove();
    overlay.remove();
  };

  registered.set(textarea, entry);
}

function unregisterTextarea(textarea: HTMLTextAreaElement): void {
  const entry = registered.get(textarea);
  if (!entry) return;
  entry.cleanup();
  registered.delete(textarea);
}

// ─── Fix popup ───────────────────────────────────────────────────────────────

function showFixPopup(edit: Edit, textarea: HTMLTextAreaElement): void {
  if (!popupContainer) {
    popupContainer = document.createElement('div');
    popupContainer.style.cssText = 'position:fixed;z-index:2147483647;';
    document.body.appendChild(popupContainer);
    popupRoot = createRoot(popupContainer);
  }

  const rect = textarea.getBoundingClientRect();
  popupRoot?.render(
    <React.StrictMode>
      <FixPopup
        edit={edit}
        originalText={textarea.value}
        textarea={textarea}
        onClose={() => popupRoot?.render(<React.StrictMode><></></React.StrictMode>)}
        style={{ top: `${rect.bottom + 8}px`, left: `${rect.left}px` }}
      />
    </React.StrictMode>
  );
}

// ─── Initial scan ────────────────────────────────────────────────────────────

function scanForTextareas(root: Document | Element = document): void {
  const textareas = root instanceof Document
    ? root.querySelectorAll<HTMLTextAreaElement>('textarea')
    : root.querySelectorAll<HTMLTextAreaElement>('textarea');
  textareas.forEach(registerTextarea);
}

// ─── MutationObserver: dynamic textareas ─────────────────────────────────────

const observer = new MutationObserver((mutations) => {
  for (const mutation of mutations) {
    for (const node of mutation.addedNodes) {
      if (!(node instanceof Element)) continue;
      if (node instanceof HTMLTextAreaElement) {
        registerTextarea(node);
      } else {
        scanForTextareas(node);
      }
    }
    for (const node of mutation.removedNodes) {
      if (node instanceof HTMLTextAreaElement) {
        unregisterTextarea(node);
      }
    }
  }
});

observer.observe(document.body, { childList: true, subtree: true });

// ─── Bootstrap ───────────────────────────────────────────────────────────────

// Register custom element before scanning
import('./HighlightOverlay').then(() => {
  scanForTextareas();
});

// ─── Keyboard shortcut handler ──────────────────────────────────────────────

chrome.runtime.onMessage.addListener((message: { type: string }) => {
  if (message.type === 'CHECK_GRAMMAR') {
    const focused = document.activeElement;
    if (focused instanceof HTMLTextAreaElement) {
      const entry = registered.get(focused);
      if (entry) {
        const store = useGrammarStore.getState();
        store.checkText(focused.value, entry.id);
      }
    }
  }
});
