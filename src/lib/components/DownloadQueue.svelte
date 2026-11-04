<script lang="ts">
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
  <div class="mt-auto shrink-0 pt-3.5">
    <Popover id="download-queue-panel" class="inset-[auto_24px_110px_auto] max-h-[min(440px,65dvh)] w-[min(350px,calc(100vw-48px))] overflow-y-auto rounded-[18px] p-3.5 shadow-[0_12px_40px_#17202a26]" bind:element={panel} onbeforetoggle={positionPanel}>
      <header class="mb-3.5 flex items-center justify-between"><h3 class="text-[0.95rem] font-bold">Downloads</h3><span class="text-xs text-[#637184]">{pending.length} na fila</span></header>
      <ul class="m-0 list-none p-0" aria-label="Fila de downloads">
        {#each $downloadQueue as item (item.key)}
          <li class="mt-2 flex items-center gap-2.5 rounded-xl border p-2.5 {item.status === 'error' ? 'border-[#e6bcc0]' : 'border-[#d8dee7]'}">
            <div class="flex h-10 w-[34px] shrink-0 items-center justify-center rounded-lg bg-[#f0f3f7] text-[#7d8a9d]" aria-hidden="true"><Icon icon={musicIcon} width="22" /></div>
            <div class="min-w-0 flex-1">
              <p class="mb-[5px] truncate text-[0.8rem] font-semibold" title={item.title}>{item.title}</p>
              <p class="text-[0.7rem] text-[#637184]">{status(item)}</p>
              {#if item.status === "downloading" && item.percent !== null}
                <progress class="h-[7px] w-full appearance-none overflow-hidden rounded-lg border-0 bg-[#e1e7f0] accent-[#eaa5ad] [&::-webkit-progress-bar]:rounded-lg [&::-webkit-progress-bar]:bg-[#e1e7f0] [&::-webkit-progress-value]:rounded-lg [&::-webkit-progress-value]:bg-[#eaa5ad]" value={item.percent} max="100" aria-label={`Progresso de ${item.title}`}></progress>
              {/if}
              {#if item.error}<p class="mt-1.5 text-[0.7rem] leading-[1.4] text-[#a51e25] [overflow-wrap:anywhere]">{item.error}</p>{/if}
            </div>
            {#if item.status === "error"}
              <div class="flex flex-col gap-1">
                <button class="flex cursor-pointer rounded-[7px] bg-[#f0f3f7] p-[5px] text-[#17202a] focus-visible:outline-3 focus-visible:outline-solid focus-visible:outline-[#7aa7ff] focus-visible:outline-offset-[3px]" type="button" onclick={() => retryDownload(item.key)} aria-label={`Tentar baixar ${item.title} novamente`} title="Tentar novamente"><Icon icon={retryIcon} width="20" /></button>
                <button class="flex cursor-pointer rounded-[7px] bg-[#f0f3f7] p-[5px] text-[#17202a] focus-visible:outline-3 focus-visible:outline-solid focus-visible:outline-[#7aa7ff] focus-visible:outline-offset-[3px]" type="button" onclick={() => removeDownload(item.key)} aria-label={`Remover ${item.title} da fila`} title="Remover da fila"><Icon icon={removeIcon} width="20" /></button>
              </div>
            {/if}
          </li>
        {/each}
      </ul>
    </Popover>
    <button class="block w-full cursor-pointer rounded-[13px] border border-[#d8dee7] bg-[#f8faff] p-3 text-left text-[#17202a] hover:bg-[#edf3ff] focus-visible:outline-3 focus-visible:outline-solid focus-visible:outline-[#7aa7ff] focus-visible:outline-offset-[3px]" type="button" popovertarget="download-queue-panel" bind:this={trigger}>
      <span class="flex items-baseline justify-between gap-2 text-[0.85rem] font-semibold">{errors === pending.length ? "Downloads com erro" : "Baixando músicas"}<small class="text-[0.65rem] font-normal text-[#637184]">{errors ? `${errors} com erro · ` : ""}{pending.length} na fila</small></span>
      <span class="mt-2.5 flex items-center gap-2.5 text-[0.8rem]"><Icon icon={menuIcon} width="22" /><progress class="h-[7px] w-full appearance-none overflow-hidden rounded-lg border-0 bg-[#e1e7f0] accent-[#eaa5ad] [&::-webkit-progress-bar]:rounded-lg [&::-webkit-progress-bar]:bg-[#e1e7f0] [&::-webkit-progress-value]:rounded-lg [&::-webkit-progress-value]:bg-[#eaa5ad]" value={percent} max="100" aria-label="Progresso total dos downloads"></progress><span>{percent}%</span></span>
    </button>
  </div>
{/if}
