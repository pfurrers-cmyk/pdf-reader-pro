<script lang="ts">
  // ========================================================================
  // TOOLBAR BUTTON — Reusable button component for the toolbar
  // ========================================================================
  import type { Snippet } from 'svelte';

  interface Props {
    title?: string;
    onclick: () => void;
    disabled?: boolean;
    active?: boolean;
    children: Snippet;
  }

  let { title = '', onclick, disabled = false, active = false, children }: Props = $props();
</script>

<button
  class="flex items-center justify-center w-7 h-7 rounded transition-colors cursor-pointer
    {active ? 'bg-blue-600 text-white' : disabled ? 'text-zinc-600 cursor-not-allowed' : 'text-zinc-400 hover:text-white hover:bg-zinc-700'}"
  {title}
  {disabled}
  onclick={onclick}
>
  {@render children()}
</button>
