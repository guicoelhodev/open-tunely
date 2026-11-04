<script lang="ts">
  import Icon from "@iconify/svelte";
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
  <div class="download-queue">
    <div id="download-queue-panel" class="queue-panel" popover="auto" bind:this={panel} onbeforetoggle={positionPanel}>
      <header><h3>Downloads</h3><span>{pending.length} na fila</span></header>
      <ul aria-label="Fila de downloads">
        {#each $downloadQueue as item (item.key)}
          <li class:failed={item.status === "error"}>
            <div class="cover" aria-hidden="true"><Icon icon={musicIcon} width="22" /></div>
            <div class="info">
              <p class="title" title={item.title}>{item.title}</p>
              <p class="status">{status(item)}</p>
              {#if item.status === "downloading" && item.percent !== null}
                <progress value={item.percent} max="100" aria-label={`Progresso de ${item.title}`}></progress>
              {/if}
              {#if item.error}<p class="error">{item.error}</p>{/if}
            </div>
            {#if item.status === "error"}
              <div class="actions">
                <button type="button" onclick={() => retryDownload(item.key)} aria-label={`Tentar baixar ${item.title} novamente`} title="Tentar novamente"><Icon icon={retryIcon} width="20" /></button>
                <button type="button" onclick={() => removeDownload(item.key)} aria-label={`Remover ${item.title} da fila`} title="Remover da fila"><Icon icon={removeIcon} width="20" /></button>
              </div>
            {/if}
          </li>
        {/each}
      </ul>
    </div>
    <button class="queue-trigger" type="button" popovertarget="download-queue-panel" bind:this={trigger}>
      <span class="label">{errors === pending.length ? "Downloads com erro" : "Baixando músicas"}<small>{errors ? `${errors} com erro · ` : ""}{pending.length} na fila</small></span>
      <span class="summary"><Icon icon={menuIcon} width="22" /><progress value={percent} max="100" aria-label="Progresso total dos downloads"></progress><span>{percent}%</span></span>
    </button>
  </div>
{/if}

<style>
  .download-queue { margin-top: auto; padding-top: 14px; flex-shrink: 0; }
  button { font: inherit; cursor: pointer; color: #17202a; border: 0; }
  button:focus-visible { outline: 3px solid #7aa7ff; outline-offset: 3px; }
  .queue-trigger { display: block; width: 100%; padding: 12px; text-align: left; border: 1px solid #d8dee7; border-radius: 13px; background: #f8faff; }
  .queue-trigger:hover { background: #edf3ff; }
  .label { display: flex; justify-content: space-between; align-items: baseline; gap: 8px; font-size: .85rem; font-weight: 600; }
  small { color: #637184; font-size: .65rem; font-weight: 400; }
  .summary { display: flex; align-items: center; gap: 10px; margin-top: 10px; font-size: .8rem; }
  progress { appearance: none; width: 100%; height: 7px; border: 0; border-radius: 8px; overflow: hidden; background: #e1e7f0; accent-color: #eaa5ad; }
  progress::-webkit-progress-bar { background: #e1e7f0; border-radius: 8px; }
  progress::-webkit-progress-value { background: #eaa5ad; border-radius: 8px; }
  .queue-panel { position: fixed; inset: auto 24px 110px auto; margin: 0; width: min(350px, calc(100vw - 48px)); max-height: min(440px, 65dvh); padding: 14px; box-sizing: border-box; overflow-y: auto; border: 1px solid #d8dee7; border-radius: 18px; background: #fff; color: #17202a; box-shadow: 0 12px 40px #17202a26; }
  header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 14px; }
  h3 { margin: 0; font-size: .95rem; }
  header span { font-size: .75rem; color: #637184; }
  ul { list-style: none; margin: 0; padding: 0; }
  li { display: flex; align-items: center; gap: 10px; padding: 10px; border: 1px solid #d8dee7; border-radius: 12px; margin-top: 8px; }
  li.failed { border-color: #e6bcc0; }
  .cover { display: flex; align-items: center; justify-content: center; width: 34px; height: 40px; flex-shrink: 0; background: #f0f3f7; color: #7d8a9d; border-radius: 8px; }
  .info { flex: 1; min-width: 0; }
  .title { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-size: .8rem; font-weight: 600; margin: 0 0 5px; }
  .status { color: #637184; font-size: .7rem; margin: 0; }
  .error { font-size: .7rem; line-height: 1.4; overflow-wrap: anywhere; color: #a51e25; margin: 6px 0 0; }
  .actions { display: flex; flex-direction: column; gap: 4px; }
  .actions button { display: flex; padding: 5px; background: #f0f3f7; border-radius: 7px; }
</style>
