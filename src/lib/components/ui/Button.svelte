<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";

  const variants = {
    primary: "bg-primary text-primary-foreground hover:bg-primary/90",
    secondary: "bg-secondary text-secondary-foreground hover:bg-secondary/80",
    soft: "bg-primary/7 text-primary hover:bg-primary/12",
    subtle: "border border-border bg-primary/7 text-primary hover:bg-primary/12",
    ghost: "bg-transparent text-primary hover:bg-primary/7",
    muted: "bg-transparent text-muted-foreground hover:bg-primary/7 hover:text-primary",
    danger: "bg-red-50 text-red-700 hover:bg-red-100 dark:bg-red-950/40 dark:text-red-300 dark:hover:bg-red-900/60",
    option: "bg-transparent text-foreground hover:bg-primary/7 aria-pressed:bg-primary/7 aria-pressed:text-primary",
    surface: "border border-border bg-muted text-card-foreground hover:bg-primary/7",
  };
  const sizes = {
    md: "px-3 py-2.5 text-sm",
    sm: "px-3 py-2 text-sm",
    icon: "size-9 p-0",
    custom: "",
  };
  const layouts = {
    inline: "inline-flex items-center justify-center gap-2",
    menu: "flex items-center gap-3 text-left",
    block: "block text-left",
  };

  let {
    children,
    element = $bindable(),
    type = "button",
    variant = "subtle",
    size = "md",
    layout = "inline",
    round = false,
    class: className = "",
    ...attributes
  }: HTMLButtonAttributes & {
    children: Snippet;
    element?: HTMLButtonElement;
    variant?: keyof typeof variants;
    size?: keyof typeof sizes;
    layout?: keyof typeof layouts;
    round?: boolean;
  } = $props();
</script>

<button
  {...attributes}
  {type}
  bind:this={element}
  class={[
    "cursor-pointer transition-colors disabled:cursor-default disabled:opacity-50 focus-visible:outline-3 focus-visible:outline-solid focus-visible:outline-ring",
    layout === "menu" ? "focus-visible:outline-offset-[-3px]" : "focus-visible:outline-offset-[3px]",
    round ? "rounded-full" : "rounded-md",
    variants[variant],
    sizes[size],
    layouts[layout],
    className,
  ]}
>
  {@render children()}
</button>
