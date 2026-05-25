<script lang="ts">
  import { activeTab, viewMode, ZOOM_PRESETS, DEFAULT_ZOOM, searchBarOpen, commandPaletteOpen, tabs, activeTabId, toolMode } from '$lib/stores';
  import { savePdfToDisk } from '$lib/api';
  import ToolbarButton from './ToolbarButton.svelte';
  import type { ViewMode } from '$lib/types';

  interface Props {
    zoom: number;
    onZoomChange: (zoom: number) => void;
    onOpenFile: () => void;
    onFitWidth: () => void;
    onFitPage: () => void;
    onToggleSidebar: () => void;
  }

  let { zoom, onZoomChange, onOpenFile, onFitWidth, onFitPage, onToggleSidebar }: Props = $props();

  function zoomIn() {
    const idx = ZOOM_PRESETS.findIndex(z => z >= zoom);
    if (idx < ZOOM_PRESETS.length - 1) onZoomChange(ZOOM_PRESETS[idx + 1]);
  }

  function zoomOut() {
    const idx = ZOOM_PRESETS.findIndex(z => z >= zoom);
    if (idx > 0) onZoomChange(ZOOM_PRESETS[idx - 1]);
  }

  function zoomReset() { onZoomChange(DEFAULT_ZOOM); }
  function setViewMode(mode: ViewMode) { viewMode.set(mode); }

  const zoomPercent = $derived(Math.round(zoom * 100));
</script>

<div class="flex items-center gap-0.5 px-2 h-9 bg-zinc-850 border-b border-zinc-700 text-zinc-300 select-none">
  <ToolbarButton title="Painel lateral" onclick={onToggleSidebar}>
    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><line x1="9" y1="3" x2="9" y2="21"/></svg>
  </ToolbarButton>
  <div class="w-px h-5 bg-zinc-700 mx-1"></div>
  <ToolbarButton title="Abrir PDF (Ctrl+O)" onclick={onOpenFile}>
    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"/></svg>
  </ToolbarButton>
  <div class="w-px h-5 bg-zinc-700 mx-1"></div>
  <ToolbarButton title="Página anterior" onclick={() => { if ($activeTab && $activeTab.currentPage > 0) activeTabId.update(id => { tabs.updateTab(id!, { currentPage: $activeTab.currentPage - 1}); return id; }); }} disabled={!$activeTab || $activeTab.currentPage === 0}>
    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
  </ToolbarButton>
  <span class="text-xs text-zinc-400 min-w-[60px] text-center font-mono">
    {#if $activeTab}{$activeTab.currentPage + 1} / {$activeTab.pageCount}{:else}– / –{/if}
  </span>
  <ToolbarButton title="Próxima página" onclick={() => { if ($activeTab && $activeTab.currentPage < $activeTab.pageCount - 1) activeTabId.update(id => { tabs.updateTab(id!, { currentPage: $activeTab.currentPage + 1}); return id; }); }} disabled={!$activeTab || ($activeTab && $activeTab.currentPage === $activeTab.pageCount - 1)}>
    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 18 15 12 9 6"/></svg>
  </ToolbarButton>
  <div class="w-px h-5 bg-zinc-700 mx-1"></div>
  <ToolbarButton title="Reduzir zoom (-)" onclick={zoomOut}>
    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/><line x1="8" y1="11" x2="14" y2="11"/></svg>
  </ToolbarButton>
  <button class="text-xs text-zinc-300 hover:text-white px-2 py-1 rounded hover:bg-zinc-700 transition-colors min-w-[50px] text-center cursor-pointer" onclick={zoomReset} title="Resetar zoom (100%)">{zoomPercent}%</button>
  <ToolbarButton title="Aumentar zoom (+)" onclick={zoomIn}>
    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/><line x1="11" y1="8" x2="11" y2="14"/><line x1="8" y1="11" x2="14" y2="11"/></svg>
  </ToolbarButton>
  <ToolbarButton title="Ajustar à largura" onclick={onFitWidth}>
    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 3H3v18h18V3z"/><path d="M3 9h18M3 15h18"/></svg>
  </ToolbarButton>
  <ToolbarButton title="Ajustar à página" onclick={onFitPage}>
    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 3H5a2 2 0 00-2 2v3m18 0V5a2 2 0 00-2-2h-3m0 18h3a2 2 0 002-2v-3M3 16v3a2 2 0 002 2h3"/></svg>
  </ToolbarButton>
  <div class="w-px h-5 bg-zinc-700 mx-1"></div>
  <ToolbarButton title="Rolagem contínua" onclick={() => setViewMode('continuous')} active={$viewMode === 'continuous'}>
    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="6" y="2" width="12" height="6" rx="1"/><rect x="6" y="9" width="12" height="6" rx="1"/><rect x="6" y="16" width="12" height="6" rx="1"/></svg>
  </ToolbarButton>
  <ToolbarButton title="Página única" onclick={() => setViewMode('single')} active={$viewMode === 'single'}>
    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="4" y="2" width="16" height="20" rx="2"/></svg>
  </ToolbarButton>
  <div class="w-px h-5 bg-zinc-700 mx-1"></div>
  
  <!-- Tools (Hand, Highlight, Sticky Note) -->
  <ToolbarButton title="Mão (Arrastar / Ler)" onclick={() => toolMode.set('hand')} active={$toolMode === 'hand'}>
    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 11V6a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v0a2 2 0 0 0-2 2v-4a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v7.5"/><path d="M14 11V9a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v7.5"/><path d="M10 11v-1.5a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v4.5"/><path d="M18 11a2 2 0 0 1 2 2v3a8 8 0 0 1-8 8h-1a8 8 0 0 1-8-8v-6a2 2 0 0 1 2-2h0a2 2 0 0 1 2 2v4"/><path d="M6 14v-4"/></svg>
  </ToolbarButton>
  <ToolbarButton title="Marca-Texto" onclick={() => toolMode.set('highlight')} active={$toolMode === 'highlight'}>
    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 19l7-7 3 3-7 7-3-3z"/><path d="M18 13l-1.5-7.5L2 2l3.5 14.5L13 18l5-5z"/><path d="M2 2l7.586 7.586"/><circle cx="11" cy="11" r="2"/></svg>
  </ToolbarButton>
  <ToolbarButton title="Nota Adesiva (Post-it)" onclick={() => toolMode.set('sticky_note')} active={$toolMode === 'sticky_note'}>
    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16c0 1.1.9 2 2 2h12a2 2 0 0 0 2-2V8l-6-6z"/><path d="M14 3v5h5M16 13H8M16 17H8M10 9H8"/></svg>
  </ToolbarButton>
  <ToolbarButton title="Salvar Anotações no Disco (Ctrl+S)" onclick={() => $activeTab && savePdfToDisk($activeTab.id)} disabled={!$activeTab}>
    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg>
  </ToolbarButton>

  <div class="flex-1"></div>
  <ToolbarButton title="Buscar (Ctrl+F)" onclick={() => searchBarOpen.set(true)}>
    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
  </ToolbarButton>
  <ToolbarButton title="Paleta de comandos (Ctrl+K)" onclick={() => commandPaletteOpen.set(true)}>
    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>
  </ToolbarButton>
</div>