<script lang="ts">
  import { globalSearchOpen, isIndexing } from '$lib/stores';
  import { indexFolder, globalSearch } from '$lib/api';
  import type { GlobalSearchResultDto } from '$lib/types';
  import { isTauri } from '$lib/api';

  // @ts-ignore
  const tauriDialog = window.__TAURI__?.dialog;

  let query = $state('');
  let results = $state<GlobalSearchResultDto[]>([]);
  let isSearching = $state(false);
  let statusMessage = $state('');

  function close() {
    globalSearchOpen.set(false);
  }

  async function handleIndexFolder() {
    if (!isTauri) {
      statusMessage = "Indexação local requer o app desktop.";
      return;
    }
    
    if (!tauriDialog) {
      statusMessage = "Erro: Plugin de diálogo não disponível.";
      return;
    }

    try {
      const selected = await tauriDialog.open({
        multiple: false,
        directory: true,
      });

      if (!selected) return;
      
      const dirPath = typeof selected === 'string' ? selected : selected[0];
      
      isIndexing.set(true);
      statusMessage = `Indexando PDFs em: ${dirPath}... (Isso pode demorar)`;
      
      const count = await indexFolder(dirPath);
      statusMessage = `Concluído. ${count} PDFs indexados com sucesso!`;
      
    } catch (e) {
      console.error(e);
      statusMessage = `Erro na indexação: ${e}`;
    } finally {
      isIndexing.set(false);
    }
  }

  async function performSearch() {
    if (!query.trim()) {
      results = [];
      return;
    }
    
    isSearching = true;
    try {
      results = await globalSearch(query);
      if (results.length === 0) {
        statusMessage = "Nenhum resultado encontrado.";
      } else {
        statusMessage = `${results.length} resultados encontrados.`;
      }
    } catch (e) {
      console.error(e);
      statusMessage = `Erro na busca: ${e}`;
    } finally {
      isSearching = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
    if (e.key === 'Enter') performSearch();
  }
</script>

{#if $globalSearchOpen}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="fixed inset-0 z-50 flex items-start justify-center pt-[15vh] bg-black/60 backdrop-blur-sm" role="dialog" aria-modal="true" aria-label="Busca Global" tabindex="-1" onclick={close} onkeydown={handleKeydown}>
    <div class="w-full max-w-2xl bg-zinc-800 rounded-xl shadow-2xl border border-zinc-700 overflow-hidden flex flex-col max-h-[70vh]" onclick={(e) => e.stopPropagation()}>
      
      <!-- Header / Search Input -->
      <div class="p-4 border-b border-zinc-700 bg-zinc-850">
        <div class="flex items-center gap-3">
          <svg class="w-5 h-5 text-blue-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path></svg>
          <input
            bind:value={query}
            type="text"
            placeholder="Buscar em todos os documentos indexados..."
            class="flex-1 bg-zinc-900 border border-zinc-700 rounded-md px-3 py-2 text-white outline-none focus:border-blue-500"
            disabled={!isTauri}
          />
          <button 
            onclick={performSearch} 
            disabled={isSearching || !isTauri || $isIndexing}
            class="bg-blue-600 hover:bg-blue-500 text-white px-4 py-2 rounded-md transition-colors disabled:opacity-50 font-medium"
          >
            {isSearching ? 'Buscando...' : 'Buscar'}
          </button>
        </div>
      </div>

      <!-- Controls & Status -->
      <div class="px-4 py-2 border-b border-zinc-750 flex items-center justify-between bg-zinc-800 text-xs">
        <button 
          onclick={handleIndexFolder} 
          disabled={$isIndexing || !isTauri}
          class="flex items-center gap-1.5 text-zinc-300 hover:text-white bg-zinc-700 hover:bg-zinc-600 px-3 py-1.5 rounded transition-colors disabled:opacity-50"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 13h6m-3-3v6m-9 1V7a2 2 0 012-2h6l2 2h6a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2z"></path></svg>
          {$isIndexing ? 'Indexando...' : 'Indexar Pasta'}
        </button>
        <span class="text-zinc-400 font-mono">{statusMessage}</span>
      </div>

      <!-- Results List -->
      <div class="flex-1 overflow-y-auto p-2 bg-zinc-900">
        {#if !isTauri}
           <div class="h-full flex flex-col items-center justify-center text-zinc-500 p-8 text-center">
              <svg class="w-12 h-12 mb-3 text-zinc-600" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"></path></svg>
              <p>O Motor de Busca Tantivy é um recurso nativo do Rust.</p>
              <p class="text-sm mt-1">Por favor, utilize o aplicativo Desktop para realizar buscas avançadas nos seus arquivos locais.</p>
           </div>
        {:else if results.length > 0}
          <div class="flex flex-col gap-2">
            {#each results as result}
              <button class="text-left w-full bg-zinc-800 hover:bg-zinc-750 border border-zinc-700 p-3 rounded-lg transition-colors cursor-pointer group">
                <div class="flex items-start justify-between mb-1">
                  <h4 class="text-white font-medium text-sm group-hover:text-blue-400 truncate">{result.title}</h4>
                  <span class="text-[10px] bg-zinc-900 text-zinc-400 px-2 py-0.5 rounded font-mono">Score: {result.score.toFixed(2)}</span>
                </div>
                <p class="text-xs text-zinc-500 mb-2 truncate" title={result.path}>{result.path}</p>
                <div class="text-sm text-zinc-300 bg-zinc-900/50 p-2 rounded border border-zinc-800/50 italic">
                  "{result.snippet}"
                </div>
              </button>
            {/each}
          </div>
        {:else if !isSearching && query}
          <div class="p-8 text-center text-zinc-500">Nenhum resultado encontrado para "{query}".</div>
        {/if}
      </div>

    </div>
  </div>
{/if}