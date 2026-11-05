<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";

  let {
    children,
    element = $bindable(),
    class: className = "",
    popover = "auto",
    ...attributes
  }: HTMLAttributes<HTMLDivElement> & {
    children: Snippet;
    element?: HTMLDivElement;
  } = $props();
</script>

<div
  {...attributes}
  bind:this={element}
  {popover}
  class="fixed m-0 border border-border bg-card text-card-foreground {className}"
>
  {@render children()}
</div>
