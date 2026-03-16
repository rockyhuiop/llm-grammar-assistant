/**
 * Position mapping utilities for the browser extension.
 * Maps character positions to pixel coordinates within a textarea.
 */

export interface TextareaMetrics {
  font: string;
  fontSize: number;
  lineHeight: number;
  paddingLeft: number;
  paddingTop: number;
  width: number;
}

/** Samples computed style metrics from a textarea element. */
export function sampleTextareaMetrics(textarea: HTMLTextAreaElement): TextareaMetrics {
  const style = window.getComputedStyle(textarea);
  return {
    font: style.font,
    fontSize: parseFloat(style.fontSize),
    lineHeight: parseFloat(style.lineHeight) || parseFloat(style.fontSize) * 1.2,
    paddingLeft: parseFloat(style.paddingLeft),
    paddingTop: parseFloat(style.paddingTop),
    width: textarea.clientWidth - parseFloat(style.paddingLeft) - parseFloat(style.paddingRight),
  };
}

export interface HighlightRect {
  top: number;
  left: number;
  width: number;
  height: number;
}

/**
 * Maps a character range (start, end indices in the textarea value)
 * to pixel rectangles suitable for absolutely-positioned highlight divs.
 *
 * Uses a hidden mirror div technique for accurate character-to-pixel mapping.
 */
export function mapPositionToRect(
  textarea: HTMLTextAreaElement,
  start: number,
  end: number
): HighlightRect[] {
  const mirror = createMirrorDiv(textarea);
  document.body.appendChild(mirror);

  try {
    const text = textarea.value;
    const beforeText = text.slice(0, start);
    const highlightText = text.slice(start, end);

    // Build mirror content with spans around the highlighted region
    const beforeSpan = document.createElement('span');
    beforeSpan.textContent = beforeText;

    const highlightSpan = document.createElement('span');
    highlightSpan.textContent = highlightText;

    mirror.appendChild(beforeSpan);
    mirror.appendChild(highlightSpan);

    const spanRect = highlightSpan.getBoundingClientRect();
    const textareaRect = textarea.getBoundingClientRect();

    // Adjust for textarea scroll position
    const scrollTop = textarea.scrollTop;

    return [
      {
        top: spanRect.top - textareaRect.top + scrollTop,
        left: spanRect.left - textareaRect.left,
        width: spanRect.width,
        height: spanRect.height,
      },
    ];
  } finally {
    document.body.removeChild(mirror);
  }
}

/** Creates a hidden mirror div that replicates a textarea's styling. */
function createMirrorDiv(textarea: HTMLTextAreaElement): HTMLDivElement {
  const style = window.getComputedStyle(textarea);
  const mirror = document.createElement('div');

  // Copy relevant styles
  const props = [
    'font', 'fontSize', 'fontFamily', 'fontWeight', 'letterSpacing',
    'lineHeight', 'padding', 'paddingLeft', 'paddingRight', 'paddingTop', 'paddingBottom',
    'borderWidth', 'borderStyle', 'wordSpacing', 'wordWrap', 'whiteSpace',
    'width', 'boxSizing',
  ];
  for (const prop of props) {
    (mirror.style as unknown as Record<string, string>)[prop] = style.getPropertyValue(prop);
  }

  mirror.style.position = 'fixed';
  mirror.style.top = '-9999px';
  mirror.style.left = '-9999px';
  mirror.style.visibility = 'hidden';
  mirror.style.overflow = 'hidden';
  mirror.style.whiteSpace = 'pre-wrap';

  return mirror;
}
