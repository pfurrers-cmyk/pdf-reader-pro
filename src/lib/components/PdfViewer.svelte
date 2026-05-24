<script lang="ts">
  // ========================================================================
  // PDF VIEWER — Renderiza páginas do PDF com rolagem suave e zoom
  // ========================================================================
  import { activeTab, viewMode, searchResults, currentSearchIndex } from '$lib/stores';
  import { renderPage } from '$lib/api';
  import type { PdfId } from '$lib/types';

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

  // Map of page_index -> base64 image data URL
  let pageImages = $state<Map<number, string>>(new Map());

  // Initialize first page image when firstPageImage changes
  $effect(() => {
    if (firstPageImage) {
      pageImages.set(0, firstPageImage);
      pageImages = new Map(pageImages);
    }
  });

  let loadingPages = $state<Set<number>>(new Set());
  let containerEl: HTMLDivElement | undefined = $state();

  // Preload visible pages + buffer
  const BUFFER = 3;

  $effect(() => {
    // When zoom or currentPage changes, preload surrounding pages
    const pagesToLoad: number[] = [];
    for (let i = Math.max(0, currentPage - BUFFER); i <= Math.min(pageCount - 1, currentPage + BUFFER); i++) {
      if (!pageImages.has(i) && !loadingPages.has(i)) {
        pagesToLoad.push(i);
      }
    }
    if (pagesToLoad.length > 0) {
      preloadPages(pagesToLoad);
    }
  });

  async function preloadPages(pageNums: number[]) {
    for (const pageNum of pageNums) {
      if (loadingPages.has(pageNum) || pageImages.has(pageNum)) continue;
      loadingPages.add(pageNum);
      loadingPages = new Set(loadingPages);

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

  // Handle scroll to detect current page
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
      >
        {#if imageSrc}
          <img
            src={imageSrc}
            alt="Página {pageIndex + 1}"
            class="block max-w-full"
            loading="lazy"
          />
        {:else if isLoading}
          <!-- Loading skeleton -->
          <div
            class="bg-zinc-800 animate-pulse rounded flex items-center justify-center"
            style:width="{800 * zoom}px"
            style:height="{1100 * zoom}px"
          >
            <svg class="w-8 h-8 text-zinc-600 animate-spin" viewBox="0 0 24 24" fill="none">
              <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" class="opacity-25"/>
              <path d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" fill="currentColor" class="opacity-75"/>
            </svg>
          </div>
        {:else}
          <!-- Placeholder -->
          <div
            class="bg-zinc-850 rounded"
            style:width="{800 * zoom}px"
            style:height="{1100 * zoom}px"
          ></div>
        {/if}

        <!-- Page number label -->
        <div class="absolute bottom-1 right-1 bg-zinc-900/80 text-zinc-500 text-[10px] px-1.5 py-0.5 rounded">
          {pageIndex + 1}
        </div>

        <!-- Search Highlights -->
        {#if $searchResults.length > 0 && $activeTab}
          {@const scale = getPageScale($activeTab.pageWidth, zoom)}
          {#each $searchResults.filter(r => r.pageIndex === pageIndex) as result, rIdx}
            {@const isCurrent = $searchResults[$currentSearchIndex] === result}
            {#each result.rects as rect}
              <div
                class="absolute mix-blend-multiply border {isCurrent ? 'bg-orange-500/50 border-orange-600' : 'bg-yellow-400/40 border-yellow-500/50'}"
                style="
                  left: {rect.left * scale}px; 
                  top: {($activeTab.pageHeight - rect.top) * scale}px; 
                  width: {Math.max((rect.right - rect.left) * scale, 4)}px; 
                  height: {Math.max((rect.top - rect.bottom) * scale, 4)}px;
                "
              ></div>
            {/each}
          {/each}
        {/if}
      </div>
    {/each}
  </div>
</div>
