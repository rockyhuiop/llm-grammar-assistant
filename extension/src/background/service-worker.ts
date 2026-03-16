/**
 * Background service worker for Chrome Manifest V3.
 * Handles extension lifecycle events and config initialization.
 */
import { loadConfig } from '../utils/storage';

// Listen for installation and set defaults
chrome.runtime.onInstalled.addListener(async ({ reason }) => {
  if (reason === 'install') {
    // loadConfig returns DEFAULT_CONFIG if nothing stored yet — persist it
    const config = await loadConfig();
    await new Promise<void>((resolve) => {
      chrome.storage.local.set({ grammar_check_config: config }, () => resolve());
    });
  }
});

// Optional: clean up credentials on uninstall
chrome.runtime.setUninstallURL('');
chrome.runtime.onSuspend?.addListener(() => {
  // No persistent state to flush — storage is already persisted
});

// Handle keyboard shortcut commands
chrome.commands.onCommand.addListener((command) => {
  if (command === 'check-grammar') {
    chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
      const tabId = tabs[0]?.id;
      if (tabId != null) {
        chrome.tabs.sendMessage(tabId, { type: 'CHECK_GRAMMAR' });
      }
    });
  }
});

export {};
