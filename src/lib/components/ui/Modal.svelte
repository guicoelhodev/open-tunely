<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLDialogAttributes } from "svelte/elements";

  let {
    children,
    element = $bindable(),
    size = "default",
    class: className = "",
    ...attributes
  }: HTMLDialogAttributes & {
    children: Snippet;
    element?: HTMLDialogElement;
    size?: "default" | "wide";
  } = $props();
</script>

<dialog
  {...attributes}
  bind:this={element}
  class="m-auto {size === 'wide' ? 'w-[min(560px,calc(100vw-32px))]' : 'w-[min(360px,calc(100vw-64px))]'} rounded-xl border border-border bg-card p-6 text-card-foreground backdrop:bg-black/40 {className}"
>
  {@render children()}
</dialog>
