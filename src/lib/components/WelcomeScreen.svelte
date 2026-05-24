<script lang="ts">
  // ========================================================================
  // WELCOME SCREEN — Tela inicial quando nenhum PDF está aberto
  // ========================================================================

  interface Props {
    onOpenFile: () => void;
  }

  let { onOpenFile }: Props = $props();

  const recentFiles: string[] = []; // TODO: load from tauri-plugin-store
</script>

<div class="flex-1 flex items-center justify-center bg-zinc-900">
  <div class="text-center max-w-md">
    <!-- Logo / Icon -->
    <div class="mb-8">
      <div class="inline-flex items-center justify-center w-20 h-20 rounded-2xl bg-gradient-to-br from-red-500 to-orange-500 shadow-lg shadow-red-500/20">
        <svg class="w-10 h-10 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
          <polyline points="14 2 14 8 20 8"/>
          <line x1="16" y1="13" x2="8" y2="13"/>
          <line x1="16" y1="17" x2="8" y2="17"/>
          <polyline points="10 9 9 9 8 9"/>
        </svg>
      </div>
    </div>

    <h1 class="text-2xl font-semibold text-white mb-2">PDF Reader Pro</h1>
    <p class="text-zinc-500 text-sm mb-8">
      Leitor de PDF ultrarrápido com busca avançada e ferramentas de edição
    </p>

    <!-- Open file button -->
    <button
      class="inline-flex items-center gap-2 px-6 py-3 bg-blue-600 hover:bg-blue-500 text-white rounded-lg
             font-medium transition-all cursor-pointer shadow-lg shadow-blue-600/20 hover:shadow-blue-500/30
             hover:scale-[1.02] active:scale-[0.98]"
      onclick={onOpenFile}
    >
      <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"/>
      </svg>
      Abrir PDF
    </button>

    <!-- Shortcuts -->
    <div class="mt-8 flex flex-wrap justify-center gap-3 text-xs text-zinc-600">
      <span class="flex items-center gap-1">
        <kbd class="px-1.5 py-0.5 bg-zinc-800 rounded text-zinc-400 text-[10px]">Ctrl+O</kbd>
        Abrir
      </span>
      <span class="flex items-center gap-1">
        <kbd class="px-1.5 py-0.5 bg-zinc-800 rounded text-zinc-400 text-[10px]">Ctrl+K</kbd>
        Comandos
      </span>
      <span class="flex items-center gap-1">
        <kbd class="px-1.5 py-0.5 bg-zinc-800 rounded text-zinc-400 text-[10px]">Ctrl+F</kbd>
        Buscar
      </span>
    </div>

    <!-- Recent files placeholder -->
    {#if recentFiles.length > 0}
      <div class="mt-8 text-left">
        <h3 class="text-xs font-medium text-zinc-500 uppercase tracking-wider mb-3">Recentes</h3>
        <div class="space-y-1">
          {#each recentFiles as file}
            <button
              class="w-full text-left px-3 py-2 text-sm text-zinc-400 hover:text-white hover:bg-zinc-800 rounded transition-colors cursor-pointer truncate"
            >
              {file}
            </button>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>
