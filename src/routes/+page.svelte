<script lang="ts">
  import { searchSong, reservePreview, resolvePreview, cancelPreview, type Song } from "$lib/services/music.service";
  import { downloadSong, downloadQueue } from "$lib/services/download.service";
  import { onDestroy, tick } from "svelte";
  import Icon from "@iconify/svelte";
  import downloadIcon from "@iconify-icons/andrsrxn/download";
  import closeIcon from "@iconify-icons/andrsrxn/x";
  import searchIcon from "@iconify-icons/andrsrxn/search";
  import playIcon from "@iconify-icons/andrsrxn/play";
  import MusicLibrary from "$lib/components/MusicLibrary.svelte";
  let localLibrary: { stop: () => void } | undefined = $state();
  let downloadError = $state("");
  let destroyed = false;
  let query = $state("");
  let song = $state<Song | null>(null);
  let loading = $state(false);
  const songQueued = $derived(!!song && $downloadQueue.some((item) => item.id === song?.id && item.status !== "completed"));
  let error = $state("");
  let previewUrl = $state("");
  let previewLoading = $state(false);
  let previewError = $state("");
  let audio: HTMLAudioElement | undefined = $state();
  let previewRequest = 0;
  let previewTicket: number | undefined;

  function resetPreview() {
    previewRequest += 1;
    if (previewTicket !== undefined) cancelPreview(previewTicket);
    previewTicket = undefined;
    audio?.pause();
    audio?.removeAttribute("src");
    audio?.load();
    previewUrl = "";
    previewLoading = false;
    previewError = "";
  }

  onDestroy(() => {
    destroyed = true;
    resetPreview();
  });

  async function preview() {
    if (!song || previewLoading) return;
    localLibrary?.stop();
    resetPreview();
    const request = previewRequest;
    const id = song.id;
    previewLoading = true;
    try {
      const ticket = await reservePreview();
      if (request !== previewRequest) {
        cancelPreview(ticket);
        return;
      }
      previewTicket = ticket;
      const url = await resolvePreview(id, ticket);
      if (request !== previewRequest) return;
      previewUrl = url;
      await tick();
      if (request !== previewRequest) return;
      previewLoading = false;
      try {
        await audio?.play();
      } catch {
        if (request === previewRequest && !previewError) {
          previewError = "A reprodução não iniciou. Use os controles abaixo ou tente novamente.";
        }
      }
    } catch (cause) {
      if (request === previewRequest) previewError = String(cause);
    } finally {
      if (request === previewRequest) previewLoading = false;
    }
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
    resetPreview();
    song = null;
    document.getElementById("song-query")?.focus();
  }

  function playbackFailed() {
    previewError = "Não foi possível reproduzir o áudio. Clique em Tentar novamente para obter um novo endereço.";
  }

  async function search(event: SubmitEvent) {
    event.preventDefault();
    if (loading || !query.trim()) return;
    loading = true;
    resetPreview();
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
<main class="min-w-0 py-6 max-[720px]:pt-0">
  <form onsubmit={search}>
    <label class="mb-2 block font-semibold" for="song-query">Música ou artista</label>
    <div class="flex flex-wrap gap-3">
      <input class="min-w-[180px] flex-1 rounded-lg border border-[#8b949e] bg-white px-4 py-3 focus-visible:outline-3 focus-visible:outline-solid focus-visible:outline-[#7aa7ff] focus-visible:outline-offset-[3px]" id="song-query" type="search" bind:value={query}
        placeholder="Ex.: Djavan — Oceano" required maxlength="300" />
      <button class="inline-flex cursor-pointer items-center justify-center rounded-lg border-0 bg-[#2458d3] px-3 py-2 text-white disabled:cursor-default disabled:opacity-60 focus-visible:outline-3 focus-visible:outline-solid focus-visible:outline-[#7aa7ff] focus-visible:outline-offset-[3px]" type="submit" disabled={loading || !query.trim()}
        aria-label={loading ? "Pesquisando…" : "Pesquisar"} title="Pesquisar">
        {#if loading}
          <span class="size-[22px] animate-spin rounded-full border-2 border-white/40 border-t-white motion-reduce:animate-none" aria-hidden="true"></span>
        {:else}
          <Icon icon={searchIcon} width="22" height="22" />
        {/if}
      </button>
    </div>
  </form>
  <div aria-live="polite" aria-busy={loading}>
    {#if error}
      <p class="my-4 leading-[1.6] text-[#a51e25]" role="alert">{error}</p>
    {:else if song}
      <article class="mt-6 rounded-xl border border-[#d8dee7] bg-white p-5">
        <div class="flex items-start justify-between gap-3">
          <p class="mb-2 text-sm leading-[1.6] text-[#536170]">Primeiro resultado do YouTube</p>
          <button class="inline-flex shrink-0 cursor-pointer items-center justify-center rounded-lg bg-transparent p-1 text-[#536170] hover:bg-[#e8edf5] focus-visible:outline-3 focus-visible:outline-solid focus-visible:outline-[#7aa7ff] focus-visible:outline-offset-[3px]" type="button" onclick={closeResult}
            aria-label="Fechar resultado do YouTube" title="Fechar resultado">
            <Icon icon={closeIcon} width="20" />
          </button>
        </div>
        <h2 class="text-[1.2rem] font-bold [overflow-wrap:anywhere]">{song.title}</h2>
        <div class="mt-5 mb-3 flex items-center gap-3">
          <button class="inline-flex cursor-pointer items-center gap-2 rounded-lg bg-[#2458d3] px-3 py-2 text-sm text-white disabled:cursor-default disabled:opacity-60 focus-visible:outline-3 focus-visible:outline-solid focus-visible:outline-[#7aa7ff] focus-visible:outline-offset-[3px]" type="button" onclick={preview} disabled={previewLoading}>
            {#if previewLoading}
              <span class="size-[18px] animate-spin rounded-full border-2 border-white/40 border-t-white motion-reduce:animate-none" aria-hidden="true"></span>
            {:else}
              <Icon icon={playIcon} width="18" />
            {/if}
            {previewError ? "Tentar novamente" : previewUrl ? "Reproduzir novamente" : "Ouvir"}
          </button>
          <button class="inline-flex cursor-pointer items-center justify-center gap-2 rounded-lg bg-[#e8edf5] px-3 py-2 text-sm text-[#536170] disabled:cursor-default disabled:opacity-60 focus-visible:outline-3 focus-visible:outline-solid focus-visible:outline-[#7aa7ff] focus-visible:outline-offset-[3px]" type="button" onclick={download} disabled={songQueued}
            aria-label={songQueued ? "Música na fila de downloads" : "Baixar música"} title="Baixar MP3">
            <Icon icon={downloadIcon} width="18" />
            Baixar MP3
          </button>
        </div>
        <div aria-live="polite" aria-busy={previewLoading}>
          {#if previewError}<p class="my-4 leading-[1.6] text-[#a51e25]" role="alert">{previewError}</p>{/if}
        </div>
        {#if previewUrl}
          {#key previewUrl}
            <audio class="mt-3 block w-full" bind:this={audio} src={previewUrl} controls preload="none"
              aria-label={`Ouvir ${song.title}`} onerror={playbackFailed}>
              <track kind="captions" />
            </audio>
          {/key}
        {/if}
      </article>
    {/if}
  </div>
  <div aria-live="polite">
    {#if downloadError}<p class="my-4 leading-[1.6] text-[#a51e25]" role="alert">{downloadError}</p>{/if}
  </div>
</main>
<MusicLibrary bind:this={localLibrary} onplay={resetPreview} />
</div>
