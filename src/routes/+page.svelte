<script lang="ts">
  import { searchSong, reservePreview, resolvePreview, cancelPreview, type Song } from "$lib/services/music.service";
  import { downloadSong, downloadQueue } from "$lib/services/download.service";
  import { onDestroy } from "svelte";
  import Icon from "@iconify/svelte";
  import downloadIcon from "@iconify-icons/andrsrxn/download";
  import closeIcon from "@iconify-icons/andrsrxn/x";
  import searchIcon from "@iconify-icons/andrsrxn/search";
  import playIcon from "@iconify-icons/andrsrxn/play";
  import MusicLibrary from "$lib/components/MusicLibrary.svelte";
  import Player from "$lib/components/Player.svelte";
  import { player, playTrack } from "$lib/services/player.service";
  let downloadError = $state("");
  let destroyed = false;
  let query = $state("");
  let song = $state<Song | null>(null);
  let loading = $state(false);
  const songQueued = $derived(!!song && $downloadQueue.some((item) => item.id === song?.id && item.status !== "completed"));
  let error = $state("");
  const previewLoading = $derived($player.loading && $player.track?.id === song?.id);
  const previewUrl = $derived($player.track?.id === song?.id ? $player.url : "");
  const previewError = $derived($player.track?.id === song?.id ? $player.error : "");
  onDestroy(() => { destroyed = true; });
  async function preview() {
    if (!song || previewLoading) return;
    const selected = song;
    await playTrack({ id: selected.id, title: selected.title, subtitle: selected.channel ?? "YouTube", load: async (signal) => {
      const ticket = await reservePreview();
      const cancel = () => cancelPreview(ticket);
      if (signal.aborted) { cancel(); signal.throwIfAborted(); }
      signal.addEventListener("abort", cancel, { once: true });
      try { signal.throwIfAborted(); return { url: await resolvePreview(selected.id, ticket), dispose: cancel }; }
      finally { signal.removeEventListener("abort", cancel); }
    } });
  }

  async function download() {
    if (!song || songQueued) return;
    const selected = song;
    downloadError = "";
    try {
      await downloadSong(selected);
    } catch (cause) {
      if (!destroyed) downloadError = String(cause);
    }
  }

  function closeResult() {
    song = null;
    document.getElementById("song-query")?.focus();
  }

  async function search(event: SubmitEvent) {
    event.preventDefault();
    if (loading || !query.trim()) return;
    loading = true;
    song = null;
    error = "";
    try {
      song = await searchSong(query);
    } catch (cause) {
      error = String(cause);
    } finally {
      loading = false;
    }
  }
</script>

<div class="mx-auto grid max-w-[1180px] grid-cols-[minmax(0,1fr)_330px] gap-8 px-6 py-8 max-[720px]:grid-cols-[minmax(0,1fr)] max-[720px]:gap-5 max-[720px]:px-4 max-[720px]:py-5">
<main class="flex min-w-0 flex-col pt-6 max-[720px]:pt-0">
  <form onsubmit={search}>
    <label class="mb-2 block font-semibold" for="song-query">Encontre sua próxima música</label>
    <div class="flex flex-wrap gap-3">
      <input class="min-w-[180px] flex-1 rounded-md border border-input bg-card px-4 py-3 focus-visible:outline-3 focus-visible:outline-solid focus-visible:outline-ring focus-visible:outline-offset-[3px]" id="song-query" type="search" bind:value={query}
        placeholder="Ex.: Djavan — Oceano" required maxlength="300" />
      <button class="inline-flex cursor-pointer items-center justify-center rounded-md border-0 bg-primary px-3 py-2 text-primary-foreground disabled:cursor-default disabled:opacity-60 focus-visible:outline-3 focus-visible:outline-solid focus-visible:outline-ring focus-visible:outline-offset-[3px]" type="submit" disabled={loading || !query.trim()}
        aria-label={loading ? "Pesquisando…" : "Pesquisar"} title="Pesquisar">
        {#if loading}
          <span class="size-[22px] animate-spin rounded-full border-2 border-primary-foreground/40 border-t-primary-foreground motion-reduce:animate-none" aria-hidden="true"></span>
        {:else}
          <Icon icon={searchIcon} width="22" height="22" />
        {/if}
      </button>
    </div>
  </form>
  <div aria-live="polite" aria-busy={loading}>
    {#if error}
      <p class="my-4 leading-[1.6] text-red-700 dark:text-red-300" role="alert">{error}</p>
    {:else if song}
      <article class="mt-6 rounded-lg border border-border bg-card p-5">
        <div class="flex items-start justify-between gap-3">
          <p class="mb-2 text-sm leading-[1.6] text-muted-foreground">Primeiro resultado do YouTube</p>
          <button class="inline-flex shrink-0 cursor-pointer items-center justify-center rounded-md bg-transparent p-1 text-muted-foreground hover:bg-secondary focus-visible:outline-3 focus-visible:outline-solid focus-visible:outline-ring focus-visible:outline-offset-[3px]" type="button" onclick={closeResult}
            aria-label="Fechar resultado do YouTube" title="Fechar resultado">
            <Icon icon={closeIcon} width="20" />
          </button>
        </div>
        <h2 title={song.title} class="text-[1.2rem] font-bold [overflow-wrap:anywhere]">{song.title}</h2>
        <p class="mt-1 text-xs text-muted-foreground">{song.channel ?? "YouTube"}</p>
        <div class="mt-5 mb-3 flex items-center gap-3">
          <button class="inline-flex cursor-pointer items-center gap-2 rounded-md bg-primary px-3 py-2 text-sm text-primary-foreground disabled:cursor-default disabled:opacity-60 focus-visible:outline-3 focus-visible:outline-solid focus-visible:outline-ring focus-visible:outline-offset-[3px]" type="button" onclick={preview} disabled={previewLoading}>
            {#if previewLoading}
              <span class="size-[18px] animate-spin rounded-full border-2 border-primary-foreground/40 border-t-primary-foreground motion-reduce:animate-none" aria-hidden="true"></span>
            {:else}
              <Icon icon={playIcon} width="18" />
            {/if}
            {previewError ? "Tentar novamente" : previewUrl ? "Reproduzir novamente" : "Ouvir"}
          </button>
          <button class="inline-flex cursor-pointer items-center justify-center gap-2 rounded-md bg-secondary px-3 py-2 text-sm text-secondary-foreground disabled:cursor-default disabled:opacity-60 focus-visible:outline-3 focus-visible:outline-solid focus-visible:outline-ring focus-visible:outline-offset-[3px]" type="button" onclick={download} disabled={songQueued}
            aria-label={songQueued ? "Música na fila de downloads" : "Baixar música"} title="Baixar MP3">
            <Icon icon={downloadIcon} width="18" />
            Baixar MP3
          </button>
        </div>
        <div aria-live="polite" aria-busy={previewLoading}>
          {#if previewError}<p class="my-4 leading-[1.6] text-red-700 dark:text-red-300" role="alert">{previewError}</p>{/if}
        </div>
      </article>
    {/if}
  </div>
  <div aria-live="polite">
    {#if downloadError}<p class="my-4 leading-[1.6] text-red-700 dark:text-red-300" role="alert">{downloadError}</p>{/if}
  </div>
<Player />
</main>
<MusicLibrary />
</div>
