<script lang="ts">
  import { searchBarOpen, searchQuery, searchResults, currentSearchIndex, activeTab, isSearching } from '$lib/stores';
  import { searchInDoc } from '$lib/api';

  let inputEl: HTMLInputElement | undefined = $state();

  $effect(() => {
    if ($searchBarOpen && inputEl) {
      inputEl.focus();
    }
  });

  async function performSearch() {
    if (!$activeTab || !$searchQuery.trim()) {
      searchResults.set([]);
      currentSearchIndex.set(0);
      return;
    }

    isSearching.set(true);
    try {
      const results = await searchInDoc($activeTab.id, $searchQuery);
      searchResults.set(results);
      currentSearchIndex.set(0);
    } catch (e) {
      console.error("Search error:", e);
    } finally {
      isSearching.set(false);
    }
  }

  function nextResult() {
    if ($searchResults.length > 0) {
      currentSearchIndex.update(n => (n + 1) % $searchResults.length);
    }
  }

  function prevResult() {
    if ($searchResults.length > 0) {
      currentSearchIndex.update(n => (n - 1 + $searchResults.length) % $searchResults.length);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      if (e.shiftKey) prevResult();
      else nextResult();
    } else if (e.key === 'Escape') {
      searchBarOpen.set(false);
    }
  }

  // Automatically search when query changes, with a small debounce
  let timeout: any;
  function onInput() {
    clearTimeout(timeout);
    timeout = setTimeout(performSearch, 500);
  }
</script>

{#if $searchBarOpen}
  <div class="absolute top-14 right-8 z-40 bg-zinc-800 border border-zinc-700 rounded-lg shadow-xl p-2 flex items-center gap-2">
    <input
      bind:this={inputEl}
      bind:value={$searchQuery}
      oninput={onInput}
      onkeydown={handleKeydown}
      type="text"
      placeholder="Buscar no documento..."
      class="bg-zinc-900 text-white text-sm rounded px-3 py-1.5 w-64 outline-none border border-zinc-700 focus:border-blue-500"
    />
    {#if $isSearching}
       <span class="text-zinc-400 text-xs w-12 text-center">...</span>
    {:else if $searchResults.length > 0}
       <span class="text-zinc-400 text-xs w-12 text-center">{$currentSearchIndex + 1}/{$searchResults.length}</span>
    {:else if $searchQuery}
       <span class="text-zinc-400 text-xs w-12 text-center">0/0</span>
    {/if}
    <div class="flex items-center border-l border-zinc-700 pl-2 ml-1">
      <button onclick={prevResult} class="p-1.5 text-zinc-400 hover:text-white hover:bg-zinc-700 rounded" title="Anterior (Shift+Enter)">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7"/></svg>
      </button>
      <button onclick={nextResult} class="p-1.5 text-zinc-400 hover:text-white hover:bg-zinc-700 rounded" title="Próxima (Enter)">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/></svg>
      </button>
      <button onclick={() => searchBarOpen.set(false)} class="p-1.5 ml-1 text-zinc-400 hover:text-white hover:bg-zinc-700 rounded" title="Fechar (Esc)">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/></svg>
      </button>
    </div>
  </div>
{/if}
