// ============================================================================
// SVELTE STORES — Global State Management for PDF Reader Pro
// ============================================================================

import { writable, derived, get } from 'svelte/store';
import type { PdfTab, ViewMode, SearchResultDto } from '$lib/types';
import type { PdfId } from '$lib/types';

// ---- Active Tabs ----
function createTabsStore() {
  const { subscribe, set, update } = writable<PdfTab[]>([]);
  
  return {
    subscribe,
    set,
    update,
    
    addTab(tab: PdfTab) {
      update(tabs => [...tabs, tab]);
    },
    
    removeTab(pdfId: PdfId) {
      update(tabs => tabs.filter(t => t.id !== pdfId));
    },
    
    updateTab(pdfId: PdfId, changes: Partial<PdfTab>) {
      update(tabs =>
        tabs.map(t => (t.id === pdfId ? { ...t, ...changes } : t))
      );
    },
    
    getTab(pdfId: PdfId): PdfTab | undefined {
      return get({ subscribe }).find(t => t.id === pdfId);
    },
  };
}

export const tabs = createTabsStore();

// ---- Active Tab ID ----
export const activeTabId = writable<PdfId | null>(null);

// ---- Derived: Current active tab ----
export const activeTab = derived(
  [tabs, activeTabId],
  ([$tabs, $activeTabId]) => {
    return $tabs.find(t => t.id === $activeTabId) ?? null;
  }
);

// ---- View Mode ----
export const viewMode = writable<ViewMode>('continuous');

// ---- Sidebar Open ----
export const sidebarOpen = writable<boolean>(false);

// ---- Search Query ----
export const searchQuery = writable<string>('');
export const searchResults = writable<SearchResultDto[]>([]);
export const currentSearchIndex = writable<number>(0);
export const isSearching = writable<boolean>(false);

// ---- Command Palette ----
export const searchBarOpen = writable<boolean>(false);
export const commandPaletteOpen = writable<boolean>(false);

// ---- Loading State ----
export const isGlobalLoading = writable<boolean>(false);

// ---- Zoom Presets ----
export const ZOOM_PRESETS = [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 3.0, 4.0] as const;
export const DEFAULT_ZOOM = 1.0;
