<script lang="ts">
  // ========================================================================
  // TAB BAR — Sistema de abas para múltiplos documentos PDF
  // ========================================================================
  import { tabs, activeTabId } from '$lib/stores';
  import { closePdf } from '$lib/api';
  import type { PdfId } from '$lib/types';

  interface Props {
    onOpenFile: () => void;
  }

  let { onOpenFile }: Props = $props();

  function switchTab(id: PdfId) {
    activeTabId.set(id);
  }

  async function closeTab(id: PdfId, event: MouseEvent) {
    event.stopPropagation();
    try {
      await closePdf(id);
      tabs.removeTab(id);
      const remaining = $tabs;
      if ($activeTabId === id) {
        activeTabId.set(remaining.length > 0 ? remaining[remaining.length - 1].id : null);
      }
    } catch (e) {
      console.error('Erro ao fechar PDF:', e);
    }
  }

  function getShortName(name: string, maxLen = 24): string {
    if (name.length <= maxLen) return name;
    const ext = name.lastIndexOf('.');
    const base = name.substring(0, ext);
    if (base.length <= maxLen - 4) return name;
    return base.substring(0, maxLen - 4) + '…' + name.substring(ext);
  }
</script>

<div class="tab-bar flex items-center bg-zinc-800 border-b border-zinc-700 h-10 select-none">
  <!-- Tabs -->
  <div class="flex items-center overflow-x-auto flex-1 min-w-0 scrollbar-hide">
    {#each $tabs as tab (tab.id)}
      <div
        class="tab-item flex items-center gap-2 px-3 h-10 text-xs whitespace-nowrap border-r border-zinc-700
               transition-colors cursor-pointer min-w-0 max-w-[200px]
               {$activeTabId === tab.id
                 ? 'bg-zinc-900 text-white border-b-2 border-b-blue-500'
                 : 'bg-zinc-800 text-zinc-400 hover:bg-zinc-750 hover:text-zinc-200'}"
        role="tab"
        tabindex="0"
        aria-selected={$activeTabId === tab.id}
        onclick={() => switchTab(tab.id)}
        onkeydown={(e) => { if (e.key === 'Enter') switchTab(tab.id); }}
      >
        <!-- PDF icon -->
        <svg class="w-3.5 h-3.5 shrink-0 text-red-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
          <polyline points="14 2 14 8 20 8"/>
          <line x1="16" y1="13" x2="8" y2="13"/>
          <line x1="16" y1="17" x2="8" y2="17"/>
          <polyline points="10 9 9 9 8 9"/>
        </svg>
        <span class="truncate">{getShortName(tab.fileName)}</span>
        <!-- Close button (span instead of button to avoid nested button) -->
        <span
          class="ml-auto p-0.5 rounded hover:bg-zinc-600 shrink-0 cursor-pointer"
          style="opacity: 0.5"
          role="button"
          tabindex="-1"
          onclick={(e) => closeTab(tab.id, e)}
          onkeydown={(e) => { if (e.key === 'Enter') closeTab(tab.id, e as any); }}
        >
          <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </span>
      </div>
    {/each}
  </div>

  <!-- Open file button -->
  <button
    class="flex items-center justify-center w-10 h-10 text-zinc-400 hover:text-white hover:bg-zinc-700 transition-colors shrink-0 cursor-pointer"
    onclick={onOpenFile}
    title="Abrir PDF (Ctrl+O)"
  >
    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <line x1="12" y1="5" x2="12" y2="19"/>
      <line x1="5" y1="12" x2="19" y2="12"/>
    </svg>
  </button>
</div>

<style>
  .scrollbar-hide::-webkit-scrollbar {
    display: none;
  }
  .scrollbar-hide {
    -ms-overflow-style: none;
    scrollbar-width: none;
  }
</style>
