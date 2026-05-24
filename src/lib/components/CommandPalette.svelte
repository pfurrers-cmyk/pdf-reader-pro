<script lang="ts">
  import { commandPaletteOpen } from '$lib/stores';

  interface Command {
    id: string;
    label: string;
    shortcut?: string;
    category: string;
    action: () => void;
  }

  interface Props { commands: Command[]; }
  let { commands }: Props = $props();

  let query = $state('');
  let selectedIndex = $state(0);
  let inputEl: HTMLInputElement | undefined = $state();

  const filteredCommands = $derived(
    commands.filter(cmd =>
      cmd.label.toLowerCase().includes(query.toLowerCase()) ||
      cmd.category.toLowerCase().includes(query.toLowerCase())
    )
  );

  $effect(() => {
    query; // depend explicitly on query
    selectedIndex = 0;
  });

  $effect(() => {
    if ($commandPaletteOpen && inputEl) {
      setTimeout(() => inputEl?.focus(), 50);
      query = '';
    }
  });

  function close() { commandPaletteOpen.set(false); }
  function executeCommand(cmd: Command) { close(); cmd.action(); }

  function handleKeydown(event: KeyboardEvent) {
    switch (event.key) {
      case 'Escape': close(); break;
      case 'ArrowDown':
        event.preventDefault();
        selectedIndex = Math.min(selectedIndex + 1, filteredCommands.length - 1);
        break;
      case 'ArrowUp':
        event.preventDefault();
        selectedIndex = Math.max(selectedIndex - 1, 0);
        break;
      case 'Enter':
        if (filteredCommands[selectedIndex]) executeCommand(filteredCommands[selectedIndex]);
        break;
    }
  }
</script>

{#if $commandPaletteOpen}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="fixed inset-0 z-50 flex items-start justify-center pt-[20vh] bg-black/50 backdrop-blur-sm" role="dialog" aria-modal="true" aria-label="Paleta de comandos" tabindex="-1" onclick={close} onkeydown={handleKeydown}>
    <div class="w-full max-w-lg bg-zinc-800 rounded-xl shadow-2xl border border-zinc-700 overflow-hidden" onclick={(e) => e.stopPropagation()}>
      <div class="flex items-center px-4 h-12 border-b border-zinc-700">
        <svg class="w-4 h-4 text-zinc-500 mr-3 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
        <input bind:this={inputEl} bind:value={query} type="text" placeholder="Digitar comando..." class="flex-1 bg-transparent text-sm text-white placeholder-zinc-500 outline-none" />
        <kbd class="text-[10px] text-zinc-600 bg-zinc-900 px-1.5 py-0.5 rounded ml-2">ESC</kbd>
      </div>
      <div class="max-h-[300px] overflow-y-auto py-1">
        {#each filteredCommands as cmd, i (cmd.id)}
          <button class="w-full flex items-center justify-between px-4 py-2.5 text-sm transition-colors cursor-pointer {i === selectedIndex ? 'bg-blue-600/20 text-white' : 'text-zinc-300 hover:bg-zinc-750'}" onclick={() => executeCommand(cmd)} onmouseenter={() => (selectedIndex = i)}>
            <div class="flex items-center gap-2"><span class="text-[10px] text-zinc-600 uppercase tracking-wider w-16">{cmd.category}</span><span>{cmd.label}</span></div>
            {#if cmd.shortcut}<kbd class="text-[10px] text-zinc-600 bg-zinc-900 px-1.5 py-0.5 rounded">{cmd.shortcut}</kbd>{/if}
          </button>
        {/each}
        {#if filteredCommands.length === 0}
          <div class="px-4 py-8 text-center text-sm text-zinc-600">Nenhum comando encontrado</div>
        {/if}
      </div>
    </div>
  </div>
{/if}