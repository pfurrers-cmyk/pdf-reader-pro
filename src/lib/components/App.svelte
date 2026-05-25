<script lang="ts">
  // ========================================================================
  // APP — Componente raiz do PDF Reader Pro
  // ========================================================================
  import {
    tabs,
    activeTabId,
    activeTab,
    commandPaletteOpen,
    sidebarOpen,
    DEFAULT_ZOOM,
  } from '$lib/stores';
  import { openPdf, openFileDialog, isTauri, searchInDoc } from '$lib/api';
  import { checkForAppUpdates } from '$lib/updater';
  import { savePdfToDisk } from '$lib/api';
  import type { PdfTab, SearchResultDto } from '$lib/types';

  import TabBar from '$lib/components/TabBar.svelte';
  import Toolbar from '$lib/components/Toolbar.svelte';
  import PdfViewer from '$lib/components/PdfViewer.svelte';
  import WelcomeScreen from '$lib/components/WelcomeScreen.svelte';
  import CommandPalette from '$lib/components/CommandPalette.svelte';
  import SearchBar from '$lib/components/SearchBar.svelte';
  import GlobalSearch from '$lib/components/GlobalSearch.svelte';

  // ---- Version Stamp (Padrão Indústria Brasileira) ----
  const APP_VERSION = '0.1.0';
  const BUILD_DATE = '24/05/2026 17:19:00 BRT';

  // ---- State ----
  let currentZoom = $state(1.0);
  let firstPageImages = $state<Map<string, string>>(new Map());

  // ---- Command Palette ----
  const commands = [
    { id: 'open', label: 'Abrir PDF', shortcut: 'Ctrl+O', category: 'arquivo', action: handleOpenFile },
    { id: 'close', label: 'Fechar aba atual', shortcut: 'Ctrl+W', category: 'arquivo', action: handleCloseCurrent },
    { id: 'zoom-in', label: 'Aumentar zoom', shortcut: 'Ctrl++', category: 'visualização', action: () => changeZoom(currentZoom + 0.25) },
    { id: 'zoom-out', label: 'Reduzir zoom', shortcut: 'Ctrl+-', category: 'visualização', action: () => changeZoom(currentZoom - 0.25) },
    { id: 'zoom-reset', label: 'Zoom 100%', shortcut: 'Ctrl+0', category: 'visualização', action: () => changeZoom(1.0) },
    { id: 'fit-width', label: 'Ajustar à largura', category: 'visualização', action: () => changeZoom(1.2) },
    { id: 'view-continuous', label: 'Rolagem contínua', category: 'visualização', action: () => {} },
    { id: 'view-single', label: 'Página única', category: 'visualização', action: () => {} },
    { id: 'view-two', label: 'Duas páginas', category: 'visualização', action: () => {} },
    { id: 'toggle-sidebar', label: 'Alternar painel lateral', category: 'visualização', action: () => $sidebarOpen = !$sidebarOpen },
    { id: 'search', label: 'Buscar no documento', shortcut: 'Ctrl+F', category: 'busca', action: () => searchBarOpen.set(true) },
    { id: 'global-search', label: 'Busca global (índice)', shortcut: 'Ctrl+Shift+F', category: 'busca', action: () => globalSearchOpen.set(true) },
  ];

  // ---- File Operations ----
  let fileInputEl: HTMLInputElement | undefined = $state();

  async function handleOpenFile() {
    if (isTauri) {
      try {
        const filePath = await openFileDialog();
        if (!filePath) return; // User cancelled
        await loadPdf(filePath);
      } catch (e) {
        console.error('Erro ao abrir arquivo nativo:', e);
      }
    } else {
      // Web fallback
      fileInputEl?.click();
    }
  }

  async function handleWebFileSelect(event: Event) {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;

    try {
      console.log('[PDF Reader Pro] Abrindo PDF WebFallback:', file.name);
      const result = await openPdf('', file);
      finishPdfLoad(result);
    } catch (e) {
      console.error('[PDF Reader Pro] Erro ao carregar PDF WebFallback:', e);
      alert("Erro ao carregar PDF na versão Web: " + e);
    } finally {
      target.value = ''; // Reset input
    }
  }

  async function loadPdf(path: string) {
    try {
      console.log('[PDF Reader Pro] Abrindo PDF nativo:', path);
      const result = await openPdf(path);
      finishPdfLoad(result);
    } catch (e) {
      console.error('[PDF Reader Pro] Erro ao carregar PDF nativo:', e);
    }
  }

  function finishPdfLoad(result: any) {
    console.log('[PDF Reader Pro] PDF carregado:', result.pdfId, result.pageCount, 'páginas');

    firstPageImages.set(result.pdfId, result.firstPageImage);
    firstPageImages = new Map(firstPageImages);

    const newTab: PdfTab = {
      id: result.pdfId,
      fileName: result.fileName,
      pageCount: result.pageCount,
      pageWidth: result.pageWidth,
      pageHeight: result.pageHeight,
      currentPage: 0,
      zoom: DEFAULT_ZOOM,
      scrollPosition: 0,
      renderedPages: new Map(),
      isLoading: false,
    };

    tabs.addTab(newTab);
    activeTabId.set(result.pdfId);
    currentZoom = DEFAULT_ZOOM;
  }

  function handleCloseCurrent() {
    const tab = $activeTab;
    if (tab) {
      tabs.removeTab(tab.id);
      firstPageImages.delete(tab.id);
      firstPageImages = new Map(firstPageImages);
      const remaining = $tabs;
      if ($activeTabId === tab.id) {
        activeTabId.set(remaining.length > 0 ? remaining[remaining.length - 1].id : null);
      }
    }
  }

  function changeZoom(newZoom: number) {
    currentZoom = Math.max(0.1, Math.min(5.0, newZoom));
  }

  function handlePageChange(page: number) {
    if ($activeTab) {
      tabs.updateTab($activeTab.id, { currentPage: page });
    }
  }

  function handleFitWidth() {
    if ($activeTab) {
      const windowWidth = window.innerWidth;
      const fitZoom = windowWidth / ($activeTab.pageWidth * 1.5);
      changeZoom(Math.round(fitZoom * 10) / 10);
    }
  }

  function handleFitPage() {
    if ($activeTab) {
      const windowHeight = window.innerHeight - 80;
      const fitZoom = windowHeight / ($activeTab.pageHeight * 1.5);
      changeZoom(Math.round(fitZoom * 10) / 10);
    }
  }

  // ---- Keyboard Shortcuts ----
  function handleGlobalKeydown(event: KeyboardEvent) {
    const ctrl = event.ctrlKey || event.metaKey;

    if (ctrl && event.key === 'k') {
      event.preventDefault();
      commandPaletteOpen.set(true);
    } else if (ctrl && event.key === 'o') {
      event.preventDefault();
      handleOpenFile();
    } else if (ctrl && event.key === 'w') {
      event.preventDefault();
      handleCloseCurrent();
    } else if (ctrl && event.key === '=') {
      event.preventDefault();
      changeZoom(currentZoom + 0.25);
    } else if (ctrl && event.key === '-') {
      event.preventDefault();
      changeZoom(currentZoom - 0.25);
    } else if (ctrl && event.key === '0') {
      event.preventDefault();
      changeZoom(1.0);
    } else if (ctrl && event.key === 's') {
      event.preventDefault();
      if ($activeTab) savePdfToDisk($activeTab.id);
    } else if (ctrl && event.shiftKey && event.key.toLowerCase() === 'f') {
      event.preventDefault();
      globalSearchOpen.set(true);
    } else if (ctrl && event.key === 'f') {
      event.preventDefault();
      searchBarOpen.set(true);
    } else if (event.key === 'Escape') {
      commandPaletteOpen.set(false);
    }
  }

  $effect(() => {
    window.addEventListener('keydown', handleGlobalKeydown);
    checkForAppUpdates();
    return () => window.removeEventListener('keydown', handleGlobalKeydown);
  });

  const activeFirstPageImage = $derived(
    $activeTab ? (firstPageImages.get($activeTab.id) || '') : ''
  );
</script>

<div class="flex flex-col h-screen w-screen overflow-hidden bg-zinc-900">
  <TabBar onOpenFile={handleOpenFile} />

  {#if $activeTab}
    <Toolbar
      zoom={currentZoom}
      onZoomChange={changeZoom}
      onOpenFile={handleOpenFile}
      onFitWidth={handleFitWidth}
      onFitPage={handleFitPage}
      onToggleSidebar={() => ($sidebarOpen = !$sidebarOpen)}
    />
  {/if}

  {#if $activeTab}
    <PdfViewer
      pdfId={$activeTab.id}
      pageCount={$activeTab.pageCount}
      currentPage={$activeTab.currentPage}
      zoom={currentZoom}
      firstPageImage={activeFirstPageImage}
      onPageChange={handlePageChange}
    />
  {:else}
    <WelcomeScreen onOpenFile={handleOpenFile} />
  {/if}

  <CommandPalette {commands} />
  <SearchBar />
  <GlobalSearch />

  <!-- Version Stamp (Padrão Indústria Brasileira) -->
  <div class="fixed bottom-1 right-2 text-[10px] text-zinc-700 select-none pointer-events-none z-50 font-mono">
    PDF Reader Pro v{APP_VERSION} — {BUILD_DATE}
  </div>

  <!-- Hidden File Input for Web Fallback -->
  {#if !isTauri}
    <input 
      type="file" 
      accept=".pdf,application/pdf"
      bind:this={fileInputEl} 
      onchange={handleWebFileSelect}
      class="hidden" 
      aria-hidden="true" 
      tabindex="-1"
    />
  {/if}
</div>
