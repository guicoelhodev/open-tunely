<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLDialogAttributes } from "svelte/elements";

  let {
    children,
    element = $bindable(),
    class: className = "",
    ...attributes
  }: HTMLDialogAttributes & {
    children: Snippet;
    element?: HTMLDialogElement;
  } = $props();
</script>

<dialog
  {...attributes}
  bind:this={element}
  class="m-auto w-[min(360px,calc(100vw-64px))] rounded-2xl border border-[#d8dee7] bg-white p-6 text-[#17202a] backdrop:bg-[#17202a55] {className}"
>
  {@render children()}
</dialog>
