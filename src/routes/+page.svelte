<script lang="ts">
  import { searchSong, reservePreview, resolvePreview, cancelPreview, type Song } from "$lib/services/music.service";
  import { downloadSong, downloadQueue } from "$lib/services/download.service";
  import { onDestroy, tick } from "svelte";
  import Icon from "@iconify/svelte";
  import downloadIcon from "@iconify-icons/andrsrxn/download";
  import closeIcon from "@iconify-icons/andrsrxn/x";
  import searchIcon from "@iconify-icons/andrsrxn/search";
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

<div class="app-layout">
<main>
  <h1>Pesquisar música</h1>
  <p>Encontre uma música no YouTube pelo nome ou artista.</p>
  <form onsubmit={search}>
    <label for="song-query">Música ou artista</label>
    <div class="search-row">
      <input id="song-query" type="search" bind:value={query}
        placeholder="Ex.: Djavan — Oceano" required maxlength="300" />
      <button class="search-button" type="submit" disabled={loading || !query.trim()}
        aria-label={loading ? "Pesquisando…" : "Pesquisar"} title="Pesquisar">
        <Icon icon={searchIcon} width="32" height="32" />
      </button>
    </div>
  </form>
  <div aria-live="polite" aria-busy={loading}>
    {#if loading}
      <p>Buscando no YouTube…</p>
    {:else if error}
      <p class="error" role="alert">{error}</p>
    {:else if song}
      <article>
        <div class="result-header">
          <p class="caption">Primeiro resultado do YouTube</p>
          <button class="close-result" type="button" onclick={closeResult}
            aria-label="Fechar resultado do YouTube" title="Fechar resultado">
            <Icon icon={closeIcon} width="20" />
          </button>
        </div>
        <h2>{song.title}</h2>
        <div class="song-actions">
          <button type="button" onclick={preview} disabled={previewLoading}>
            {previewLoading ? "Preparando áudio…" : previewError ? "Tentar novamente" : previewUrl ? "Atualizar áudio" : "Ouvir"}
          </button>
          <button class="download-button" type="button" onclick={download} disabled={songQueued}
            aria-label={songQueued ? "Música na fila de downloads" : "Baixar música"} title="Baixar MP3">
            <Icon icon={downloadIcon} width="22" />
          </button>
        </div>
        <div aria-live="polite" aria-busy={previewLoading}>
          {#if previewLoading}<p class="caption">Obtendo o áudio para reprodução…</p>{/if}
          {#if previewError}<p class="error" role="alert">{previewError}</p>{/if}
        </div>
        {#if previewUrl}
          {#key previewUrl}
            <audio bind:this={audio} src={previewUrl} controls preload="none"
              aria-label={`Ouvir ${song.title}`} onerror={playbackFailed}>
              <track kind="captions" />
            </audio>
          {/key}
        {/if}
      </article>
    {/if}
  </div>
  <div aria-live="polite">
    {#if downloadError}<p class="error" role="alert">{downloadError}</p>{/if}
  </div>
</main>
<MusicLibrary bind:this={localLibrary} onplay={resetPreview} />
</div>

<style>
  :global(body) { margin: 0; background: #f6f6f6; color: #17202a; font-family: Inter, system-ui, sans-serif; }
  .app-layout { display: grid; grid-template-columns: minmax(0, 1fr) 330px; gap: 32px; max-width: 1180px; margin: 0 auto; padding: 32px 24px; }
  main { min-width: 0; padding: 24px 0; }
  h1 { margin-bottom: 8px; }
  p { line-height: 1.6; }
  form { margin-top: 32px; }
  label { display: block; margin-bottom: 8px; font-weight: 600; }
  .search-row { display: flex; gap: 12px; flex-wrap: wrap; }
  .search-button { display: inline-flex; align-items: center; justify-content: center; padding: 8px 12px; }
  input, button { font: inherit; padding: 12px 16px; border-radius: 8px; }
  input { flex: 1; min-width: 180px; border: 1px solid #8b949e; background: white; }
  button { border: 0; background: #2458d3; color: white; cursor: pointer; }
  button:disabled { opacity: .6; cursor: default; }
  input:focus-visible, button:focus-visible { outline: 3px solid #7aa7ff; outline-offset: 3px; }
  article { margin-top: 24px; padding: 20px; border: 1px solid #d8dee7; border-radius: 12px; background: white; }
  .result-header { display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; }
  .close-result { display: inline-flex; align-items: center; justify-content: center; flex-shrink: 0; padding: 4px; background: transparent; color: #1599c8; }
  .close-result:hover { background: #edf8fc; }
  .caption { margin: 0 0 8px; color: #536170; font-size: .875rem; }
  h2 { margin: 0; font-size: 1.2rem; overflow-wrap: anywhere; }
  .song-actions { display: flex; align-items: center; gap: 12px; margin: 20px 0 12px; }
  .download-button { display: inline-flex; align-items: center; justify-content: center; background: #e8edf5; color: #536170; }
  audio { display: block; width: 100%; margin-top: 12px; }
  .error { color: #a51e25; }
  @media (max-width: 720px) { .app-layout { grid-template-columns: minmax(0, 1fr); gap: 20px; padding: 20px 16px; } main { padding-top: 0; } }
</style>
