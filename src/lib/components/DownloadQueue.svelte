<script lang="ts">
  import Button from "./ui/Button.svelte";
  import Icon from "@iconify/svelte";
  import Popover from "./ui/Popover.svelte";
  import menuIcon from "@iconify-icons/andrsrxn/menu";
  import musicIcon from "@iconify-icons/andrsrxn/music-note";
  import retryIcon from "@iconify-icons/andrsrxn/reload";
  import removeIcon from "@iconify-icons/andrsrxn/x";
  import { downloadQueue, retryDownload, removeDownload, type DownloadItem } from "$lib/services/download.service";

  const pending = $derived($downloadQueue.filter((item) => item.status !== "completed"));
  const errors = $derived(pending.filter((item) => item.status === "error").length);
  const percent = $derived(Math.floor($downloadQueue.reduce((sum, item) =>
    sum + (item.status === "completed" ? 100 : Math.min(item.percent ?? 0, 99)), 0) / Math.max(1, $downloadQueue.length)));
  let trigger: HTMLButtonElement | undefined = $state();
  let panel: HTMLDivElement | undefined = $state();

  function status(item: DownloadItem) {
    if (item.status === "queued") return "Na fila";
    if (item.status === "converting") return "Convertendo para MP3…";
    if (item.status === "completed") return "Concluído";
    if (item.status === "error") return "Falha no download";
    return item.percent === null ? "Preparando download…" : `${Math.floor(item.percent)}%`;
  }

  function positionPanel(event: ToggleEvent) {
    if (event.newState !== "open" || !trigger || !panel) return;
    const bounds = trigger.getBoundingClientRect();
    panel.style.right = `${Math.max(8, window.innerWidth - bounds.right)}px`;
    panel.style.bottom = `${Math.max(8, window.innerHeight - bounds.top + 10)}px`;
    panel.style.maxHeight = `${Math.min(440, Math.max(120, bounds.top - 20))}px`;
  }

  function close(event: KeyboardEvent) {
    if (event.key === "Escape" && panel?.matches(":popover-open")) {
      panel.hidePopover();
      trigger?.focus();
    }
  }
</script>

<svelte:window onkeydown={close} />

{#if pending.length}
  <div class="mt-auto shrink-0 pt-3.5 pr-4">
    <Popover id="download-queue-panel" class="inset-[auto_24px_110px_auto] max-h-[min(440px,65dvh)] w-[min(350px,calc(100vw-48px))] overflow-y-auto rounded-xl p-3.5 shadow-[0_12px_40px_#17202a26]" bind:element={panel} onbeforetoggle={positionPanel}>
      <header class="mb-3.5 flex items-center justify-between"><h3 class="text-[0.95rem] font-bold">Downloads</h3><span class="text-xs text-muted-foreground">{pending.length} na fila</span></header>
      <ul class="m-0 list-none p-0" aria-label="Fila de downloads">
        {#each $downloadQueue as item (item.key)}
          <li class="mt-2 flex items-center gap-2.5 rounded-lg border p-2.5 {item.status === 'error' ? 'border-red-300 dark:border-red-900' : 'border-border'}">
            <div class="flex h-10 w-[34px] shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground" aria-hidden="true"><Icon icon={musicIcon} width="22" /></div>
            <div class="min-w-0 flex-1">
              <p class="mb-[5px] truncate text-[0.8rem] font-semibold" title={item.title}>{item.title}</p>
              <p class="text-[0.7rem] text-muted-foreground">{status(item)}</p>
              {#if item.status === "downloading" && item.percent !== null}
                <progress class="h-[7px] w-full appearance-none overflow-hidden rounded-lg border-0 bg-secondary accent-primary [&::-webkit-progress-bar]:rounded-lg [&::-webkit-progress-bar]:bg-secondary [&::-webkit-progress-value]:rounded-lg [&::-webkit-progress-value]:bg-primary" value={item.percent} max="100" aria-label={`Progresso de ${item.title}`}></progress>
              {/if}
              {#if item.error}<p class="mt-1.5 text-[0.7rem] leading-[1.4] text-red-700 dark:text-red-300 [overflow-wrap:anywhere]">{item.error}</p>{/if}
            </div>
            {#if item.status === "error"}
              <div class="flex flex-col justify-evenly gap-1 self-stretch">
                <Button variant="muted" size="custom" class="p-[5px]" type="button" onclick={() => retryDownload(item.key)} aria-label={`Tentar baixar ${item.title} novamente`} title="Tentar novamente"><Icon icon={retryIcon} width="20" /></Button>
                <Button variant="muted" size="custom" class="p-[5px]" type="button" onclick={() => removeDownload(item.key)} aria-label={`Remover ${item.title} da fila`} title="Remover da fila"><Icon icon={removeIcon} width="20" /></Button>
              </div>
            {/if}
          </li>
        {/each}
      </ul>
    </Popover>
    <Button variant="surface" size="custom" layout="block" class="w-full p-3" type="button" popovertarget="download-queue-panel" bind:element={trigger}>
      <span class="flex items-baseline justify-between gap-2 text-[0.85rem] font-semibold">{errors === pending.length ? "Downloads com erro" : "Baixando músicas"}<small class="text-[0.65rem] font-normal text-muted-foreground">{errors ? `${errors} com erro · ` : ""}{pending.length} na fila</small></span>
      <span class="mt-2.5 flex items-center gap-2.5 text-[0.8rem]"><Icon icon={menuIcon} width="22" /><progress class="h-[7px] w-full appearance-none overflow-hidden rounded-lg border-0 bg-secondary accent-primary [&::-webkit-progress-bar]:rounded-lg [&::-webkit-progress-bar]:bg-secondary [&::-webkit-progress-value]:rounded-lg [&::-webkit-progress-value]:bg-primary" value={percent} max="100" aria-label="Progresso total dos downloads"></progress><span>{percent}%</span></span>
    </Button>
  </div>
{/if}
