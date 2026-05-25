<script lang="ts">
  import { activeTab, viewMode, searchResults, currentSearchIndex, toolMode } from '$lib/stores';
  import { renderPage, addHighlightAnnotation, addTextAnnotation } from '$lib/api';
  import type { PdfId } from '$lib/types';
  import { untrack } from 'svelte';

  interface Props {
    pdfId: PdfId;
    pageCount: number;
    currentPage: number;
    zoom: number;
    firstPageImage: string;
    onPageChange: (page: number) => void;
  }

  let {
    pdfId,
    pageCount,
    currentPage,
    zoom,
    firstPageImage,
    onPageChange,
  }: Props = $props();

  let pageImages = $state<Map<number, string>>(new Map());
  let loadingPages = $state<Set<number>>(new Set());
  let containerEl: HTMLDivElement | undefined = $state();

  // Highlight State
  let isSelecting = $state(false);
  let selectionStart = $state<{x: number, y: number, page: number} | null>(null);
  let selectionCurrent = $state<{x: number, y: number} | null>(null);

  // Sticky Note State
  let activeStickyNote = $state<{x: number, y: number, page: number} | null>(null);
  let stickyNoteContent = $state('');
  let stickyInputEl: HTMLTextAreaElement | undefined = $state();

  $effect(() => {
    if (firstPageImage) {
      untrack(() => {
        pageImages.set(0, firstPageImage);
        pageImages = new Map(pageImages);
      });
    }
  });

  const BUFFER = 3;

  $effect(() => {
    // Only explicitly depend on zoom and currentPage to trigger this effect
    zoom; currentPage;
    
    untrack(() => {
      const pagesToLoad: number[] = [];
      for (let i = Math.max(0, currentPage - BUFFER); i <= Math.min(pageCount - 1, currentPage + BUFFER); i++) {
        if (!pageImages.has(i) && !loadingPages.has(i)) {
          pagesToLoad.push(i);
        }
      }
      
      if (pagesToLoad.length > 0) {
        // Mark as loading synchronously to prevent double-queuing
        for (const p of pagesToLoad) {
           loadingPages.add(p);
        }
        loadingPages = new Set(loadingPages);
        preloadPages(pagesToLoad);
      }
    });
  });

  async function preloadPages(pageNums: number[]) {
    for (const pageNum of pageNums) {
      try {
        const result = await renderPage(pdfId, pageNum, zoom);
        pageImages.set(pageNum, result.image);
        pageImages = new Map(pageImages);
      } catch (e) {
        console.error(`Erro ao renderizar página ${pageNum}:`, e);
      } finally {
        loadingPages.delete(pageNum);
        loadingPages = new Set(loadingPages);
      }
    }
  }

  function handleScroll() {
    if (!containerEl) return;
    const scrollTop = containerEl.scrollTop;
    const viewportHeight = containerEl.clientHeight;

    let accumulatedHeight = 0;
    const pageGap = 8;
    for (let i = 0; i < pageCount; i++) {
      const estimatedHeight = viewportHeight * 1.2;
      const pageStart = accumulatedHeight;
      const pageEnd = accumulatedHeight + estimatedHeight;

      if (scrollTop + viewportHeight / 2 >= pageStart && scrollTop + viewportHeight / 2 < pageEnd) {
        if (i !== currentPage) {
          onPageChange(i);
        }
        break;
      }
      accumulatedHeight = pageEnd + pageGap;
    }
  }

  function getImageSrc(pageIndex: number): string {
    const base64 = pageImages.get(pageIndex);
    if (base64) {
      return `data:image/png;base64,${base64}`;
    }
    return '';
  }

  function getPageScale(pageWidth: number, currentZoom: number) {
    return (800 * currentZoom) / pageWidth;
  }

  // --- Interaction Handlers ---

  function handlePointerDown(e: PointerEvent, pageIndex: number, scale: number) {
    if ($toolMode === 'hand') return;
    
    const rect = (e.currentTarget as HTMLDivElement).getBoundingClientRect();
    const x = (e.clientX - rect.left) / scale;
    const y = (e.clientY - rect.top) / scale;

    if ($toolMode === 'highlight') {
      isSelecting = true;
      selectionStart = { x, y, page: pageIndex };
      selectionCurrent = { x, y };
      (e.currentTarget as HTMLDivElement).setPointerCapture(e.pointerId);
    } else if ($toolMode === 'sticky_note') {
      activeStickyNote = { x, y, page: pageIndex };
      stickyNoteContent = '';
      setTimeout(() => stickyInputEl?.focus(), 10);
    }
  }

  function handlePointerMove(e: PointerEvent, pageIndex: number, scale: number) {
    if (!isSelecting || $toolMode !== 'highlight' || !selectionStart || selectionStart.page !== pageIndex) return;
    const rect = (e.currentTarget as HTMLDivElement).getBoundingClientRect();
    selectionCurrent = { 
      x: Math.max(0, (e.clientX - rect.left) / scale), 
      y: Math.max(0, (e.clientY - rect.top) / scale) 
    };
  }

  async function handlePointerUp(e: PointerEvent, pageIndex: number, scale: number) {
    if (!isSelecting || $toolMode !== 'highlight' || !selectionStart || selectionStart.page !== pageIndex) return;
    isSelecting = false;
    (e.currentTarget as HTMLDivElement).releasePointerCapture(e.pointerId);

    if (selectionCurrent && $activeTab) {
      const left = Math.min(selectionStart.x, selectionCurrent.x);
      const right = Math.max(selectionStart.x, selectionCurrent.x);
      const topCss = Math.min(selectionStart.y, selectionCurrent.y);
      const bottomCss = Math.max(selectionStart.y, selectionCurrent.y);

      // Convert CSS top-left origin back to PDFium bottom-left origin
      const pdfTop = $activeTab.pageHeight - topCss;
      const pdfBottom = $activeTab.pageHeight - bottomCss;

      if (right - left > 5 && pdfTop - pdfBottom > 5) {
        try {
          await addHighlightAnnotation(
            pdfId, pageIndex, 
            left, pdfBottom, right, pdfTop, 
            { r: 250, g: 204, b: 21 } // yellow-400
          );
          // Rerender page to show new annotation
          const result = await renderPage(pdfId, pageIndex, zoom);
          pageImages.set(pageIndex, result.image);
          pageImages = new Map(pageImages);
        } catch (err) {
          console.error("Erro ao adicionar highlight:", err);
        }
      }
    }
    selectionStart = null;
    selectionCurrent = null;
  }

  async function saveStickyNote() {
    if (!activeStickyNote || !$activeTab || !stickyNoteContent.trim()) {
      activeStickyNote = null;
      return;
    }

    const { x, y, page } = activeStickyNote;
    const pdfY = $activeTab.pageHeight - y;

    try {
      await addTextAnnotation(
        pdfId, page, 
        x, pdfY, 
        stickyNoteContent, 
        { r: 59, g: 130, b: 246 } // blue-500
      );
      // Rerender page
      const result = await renderPage(pdfId, page, zoom);
      pageImages.set(page, result.image);
      pageImages = new Map(pageImages);
    } catch (err) {
      console.error("Erro ao salvar sticky note:", err);
    }
    
    activeStickyNote = null;
  }

  const pageRange = $derived(Array.from({ length: pageCount }, (_, i) => i));
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  bind:this={containerEl}
  class="flex-1 overflow-y-auto overflow-x-hidden bg-zinc-900 flex flex-col items-center"
  onscroll={handleScroll}
>
  <div class="flex flex-col items-center py-4 gap-2 w-full" style="min-width: fit-content;">
    {#each pageRange as pageIndex (pageIndex)}
      {@const imageSrc = getImageSrc(pageIndex)}
      {@const isLoading = loadingPages.has(pageIndex)}
      <div 
        class="relative shadow-lg rounded-sm transition-all duration-150"
        style="cursor: {$toolMode === 'hand' ? 'grab' : ($toolMode === 'highlight' ? 'crosshair' : 'cell')}"
        onpointerdown={(e) => $activeTab && handlePointerDown(e, pageIndex, getPageScale($activeTab.pageWidth, zoom))}
        onpointermove={(e) => $activeTab && handlePointerMove(e, pageIndex, getPageScale($activeTab.pageWidth, zoom))}
        onpointerup={(e) => $activeTab && handlePointerUp(e, pageIndex, getPageScale($activeTab.pageWidth, zoom))}
      >
        {#if imageSrc}
          <img src={imageSrc} alt="Página {pageIndex + 1}" class="block max-w-full pointer-events-none" loading="lazy" />
        {:else if isLoading}
          <div class="bg-zinc-800 animate-pulse rounded flex items-center justify-center" style:width="{800 * zoom}px" style:height="{1100 * zoom}px">
            <svg class="w-8 h-8 text-zinc-600 animate-spin" viewBox="0 0 24 24" fill="none"><circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" class="opacity-25"/><path d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" fill="currentColor" class="opacity-75"/></svg>
          </div>
        {:else}
          <div class="bg-zinc-850 rounded" style:width="{800 * zoom}px" style:height="{1100 * zoom}px"></div>
        {/if}

        <div class="absolute bottom-1 right-1 bg-zinc-900/80 text-zinc-500 text-[10px] px-1.5 py-0.5 rounded">
          {pageIndex + 1}
        </div>

        {#if $searchResults.length > 0 && $activeTab}
          {@const scale = getPageScale($activeTab.pageWidth, zoom)}
          {#each $searchResults.filter(r => r.pageIndex === pageIndex) as result}
            {@const isCurrent = $searchResults[$currentSearchIndex] === result}
            {#each result.rects as rect}
              <div
                class="absolute mix-blend-multiply border pointer-events-none {isCurrent ? 'bg-orange-500/50 border-orange-600' : 'bg-yellow-400/40 border-yellow-500/50'}"
                style="left: {rect.left * scale}px; top: {($activeTab.pageHeight - rect.top) * scale}px; width: {Math.max((rect.right - rect.left) * scale, 4)}px; height: {Math.max((rect.top - rect.bottom) * scale, 4)}px;"
              ></div>
            {/each}
          {/each}
        {/if}

        <!-- Active Selection Box for Highlighting -->
        {#if isSelecting && selectionStart && selectionCurrent && selectionStart.page === pageIndex}
           <div 
             class="absolute border border-blue-500 bg-blue-500/20 pointer-events-none"
             style="
               left: {Math.min(selectionStart.x, selectionCurrent.x) * getPageScale($activeTab!.pageWidth, zoom)}px;
               top: {Math.min(selectionStart.y, selectionCurrent.y) * getPageScale($activeTab!.pageWidth, zoom)}px;
               width: {Math.abs(selectionCurrent.x - selectionStart.x) * getPageScale($activeTab!.pageWidth, zoom)}px;
               height: {Math.abs(selectionCurrent.y - selectionStart.y) * getPageScale($activeTab!.pageWidth, zoom)}px;
             "
           ></div>
        {/if}

        <!-- Active Sticky Note Input Overlay -->
        {#if activeStickyNote && activeStickyNote.page === pageIndex}
           <div 
             class="absolute z-50 p-2 bg-yellow-100 shadow-xl border border-yellow-300 rounded shadow-black/20"
             style="
               left: {activeStickyNote.x * getPageScale($activeTab!.pageWidth, zoom)}px;
               top: {activeStickyNote.y * getPageScale($activeTab!.pageWidth, zoom)}px;
               width: 200px;
             "
             onpointerdown={(e) => e.stopPropagation()}
           >
             <div class="flex justify-between items-center mb-1 border-b border-yellow-200 pb-1">
               <span class="text-[10px] font-bold text-yellow-800 uppercase tracking-wider">Nova Nota</span>
               <button class="text-yellow-600 hover:text-red-500" onclick={() => activeStickyNote = null}>
                 <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
               </button>
             </div>
             <textarea 
               bind:this={stickyInputEl}
               bind:value={stickyNoteContent}
               placeholder="Escreva sua anotação..."
               class="w-full h-24 bg-transparent resize-none outline-none text-xs text-yellow-900 placeholder:text-yellow-600/50"
               onkeydown={(e) => { if (e.key === 'Enter' && e.ctrlKey) saveStickyNote(); }}
             ></textarea>
             <div class="flex justify-end mt-1">
               <button 
                 onclick={saveStickyNote}
                 class="bg-blue-500 hover:bg-blue-600 text-white text-[10px] px-2 py-1 rounded shadow"
               >
                 Salvar (Ctrl+Enter)
               </button>
             </div>
           </div>
        {/if}

      </div>
    {/each}
  </div>
</div>