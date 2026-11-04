<script lang="ts">
  import DownloadQueue from "./DownloadQueue.svelte";
  import Icon from "@iconify/svelte";
  import folderIcon from "@iconify-icons/andrsrxn/folder-in";
  import playIcon from "@iconify-icons/andrsrxn/play";
  import pauseIcon from "@iconify-icons/andrsrxn/pause";
  import musicIcon from "@iconify-icons/andrsrxn/music-note";
  import { library, choosingFolder, chooseMusicFolder, initializeMusicLibrary, type Track } from "$lib/services/library.service";
  import { downloading, downloadNotice } from "$lib/services/download.service";
  import { loadLocalAudio, type LocalAudioSource } from "$lib/services/local-audio.service";
  import { onMount, onDestroy, tick, untrack } from "svelte";

  let { onplay }: { onplay: () => void } = $props();
  const folderLabel = $derived.by(() => {
    if (!$library) return "Selecione sua pasta de músicas";
    const parts = $library.folder.split(/[\\/]+/).filter(Boolean);
    return `${parts.length > 2 ? ".../" : ""}${parts.slice(-2).join("/")}`;
  });
  let error = $state("");
  let active = $state<Track | null>(null);
  let playing = $state(false);
  let audio = $state<HTMLAudioElement>();
  let playbackUrl = $state("");
  let preparing = $state<string | null>(null);
  let source: LocalAudioSource | undefined;
  let controller: AbortController | undefined;
  let request = 0;
  let destroyed = false;

  export function stop() {
    request += 1;
    controller?.abort();
    controller = undefined;
    audio?.pause();
    audio?.removeAttribute("src");
    audio?.load();
    source?.dispose();
    source = undefined;
    playbackUrl = "";
    preparing = null;
    active = null;
    playing = false;
  }

  onMount(() => {
    void initializeMusicLibrary().catch((cause) => {
      if (!destroyed) error = String(cause);
    });
  });

  onDestroy(() => {
    destroyed = true;
    stop();
  });

  let previousFolder: string | undefined;
  $effect(() => {
    const folder = $library?.folder;
    if (folder !== previousFolder) {
      previousFolder = folder;
      untrack(stop);
    }
  });

  async function chooseFolder() {
    error = "";
    try {
      const selected = await chooseMusicFolder();
      if (!destroyed && selected) stop();
    } catch (cause) {
      if (!destroyed) error = String(cause);
    }
  }

  async function toggle(track: Track) {
    if (active?.path === track.path && playing) {
      audio?.pause();
      return;
    }
    if (preparing === track.path) return;
    const needsSource = active?.path !== track.path || !playbackUrl || !!audio?.error;
    if (needsSource) stop();
    const current = ++request;
    error = "";
    onplay();
    try {
      if (needsSource) {
        active = track;
        preparing = track.path;
        const pending = new AbortController();
        controller = pending;
        const loaded = await loadLocalAudio(track.path, pending.signal);
        if (current !== request || destroyed) {
          loaded.dispose();
          return;
        }
        source = loaded;
        playbackUrl = loaded.url;
        await tick();
      }
      if (current !== request || destroyed) return;
      await audio?.play();
    } catch (cause) {
      if (current === request && !destroyed) {
        error ||= playbackUrl
          ? "Não foi possível iniciar a reprodução. Use os controles abaixo ou tente novamente."
          : cause instanceof Error ? cause.message : "Não foi possível ler este MP3. Tente novamente.";
      }
    } finally {
      if (current === request && !destroyed) {
        preparing = null;
        controller = undefined;
      }
    }
  }

  function started(event: Event) {
    if (event.currentTarget !== audio || !playbackUrl) return;
    playing = true;
    error = "";
    onplay();
  }

  function paused(event: Event) {
    if (event.currentTarget === audio) playing = false;
  }

  function failed(event: Event) {
    if (event.currentTarget !== audio || !playbackUrl) return;
    playing = false;
    console.warn("Falha na reprodução do MP3", { code: audio?.error?.code, message: audio?.error?.message });
    error = "O player do aplicativo não conseguiu reproduzir este MP3. Tente novamente ou selecione a pasta de novo.";
  }

  function fileSize(bytes: number) {
    return `${(bytes / 1024 / 1024).toLocaleString("pt-BR", { maximumFractionDigits: 1 })} MB`;
  }
</script>

<aside aria-label="Biblioteca de músicas" class="library">
  <header>
    <div class="folder-info">
      <h2>Minha biblioteca</h2>
      <p class="folder-path" title={$library?.folder}>
        {folderLabel}
      </p>
    </div>
    <button class="folder-button" type="button" onclick={chooseFolder} disabled={$choosingFolder || $downloading}
      aria-label="Selecionar pasta de músicas" title="Selecionar pasta de músicas">
      <Icon icon={folderIcon} width="26" />
    </button>
  </header>

  <div class="library-status" aria-live="polite" aria-busy={$choosingFolder || !!preparing}>
    {#if $choosingFolder}<p>Carregando pasta…</p>{/if}
    {#if preparing}<p>Carregando MP3…</p>{/if}
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    {#if $downloadNotice}<p class="error" role="alert">{$downloadNotice}</p>{/if}
  </div>

  {#if $library}
    <p class="track-count">{$library.tracks.length} {$library.tracks.length === 1 ? "música" : "músicas"}</p>
    {#if $library.tracks.length}
      <ul>
        {#each $library.tracks as track (track.path)}
          <li class:active={active?.path === track.path}>
            <div class="cover" aria-hidden="true"><Icon icon={musicIcon} width="24" /></div>
            <div class="track-info">
              <h3 title={track.filename}>{track.title}</h3>
              <p>MP3 <span aria-hidden="true">·</span> {fileSize(track.size)}</p>
            </div>
            <button class="play-button" type="button" onclick={() => toggle(track)}
              disabled={preparing === track.path}
              aria-label={`${active?.path === track.path && playing ? "Pausar" : "Ouvir"} ${track.title}`}
              title={active?.path === track.path && playing ? "Pausar" : "Ouvir"}>
              <Icon icon={active?.path === track.path && playing ? pauseIcon : playIcon} width="20" />
            </button>
          </li>
        {/each}
      </ul>
    {:else}
      <div class="empty-state">
        <Icon icon={musicIcon} width="32" />
        <p>Nenhum arquivo MP3 nesta pasta.</p>
        <span>Escolha outra pasta pelo ícone acima.</span>
      </div>
    {/if}
  {:else if !$choosingFolder}
    <div class="empty-state">
      <Icon icon={folderIcon} width="36" />
      <p>Suas músicas, por aqui.</p>
      <span>Selecione uma pasta para listar e ouvir seus arquivos MP3.</span>
    </div>
  {/if}

  {#if active && playbackUrl}
    <div class="local-player">
      <p title={active.title}>Tocando: {active.title}</p>
      {#key playbackUrl}
        <audio bind:this={audio} src={playbackUrl} controls preload="metadata"
          aria-label={`Reproduzir ${active.title}`} onplay={started} onpause={paused}
          onended={paused} onerror={failed}>
          <track kind="captions" />
        </audio>
      {/key}
    </div>
  {/if}
  <DownloadQueue />
</aside>

<style>
  .library { display: flex; flex-direction: column; min-width: 0; min-height: 440px; height: calc(100dvh - 64px); box-sizing: border-box; border: 1px solid #d8dee7; border-radius: 24px; background: #fff; padding: 20px 16px; }
  header { display: flex; align-items: center; gap: 12px; padding: 0 4px 18px; border-bottom: 1px solid #edf0f5; }
  .folder-info, .track-info { flex: 1; min-width: 0; }
  h2 { margin: 0 0 8px; font-size: 1rem; }
  .folder-path { margin: 0; font-family: ui-monospace, monospace; font-size: .8rem; line-height: 1.5; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; color: #536170; }
  button { display: inline-flex; align-items: center; justify-content: center; flex-shrink: 0; width: 40px; height: 40px; padding: 0; border: 0; border-radius: 10px; background: #edf3ff; color: #2458d3; cursor: pointer; }
  button:hover { background: #dce8ff; }
  button:disabled { opacity: .5; cursor: wait; }
  button:focus-visible { outline: 3px solid #7aa7ff; outline-offset: 3px; }
  .folder-button { background: transparent; color: #1599c8; }
  .folder-button:hover { background: #edf8fc; }
  .library-status p { font-size: .85rem; line-height: 1.5; }
  .error { color: #a51e25; }
  .track-count { margin: 18px 4px 12px; color: #637184; font-size: .75rem; }
  ul { flex: 1; min-height: 0; overflow-y: auto; list-style: none; padding: 2px 4px 8px; margin: 0 -4px; }
  li { display: flex; align-items: center; gap: 10px; border: 1px solid #d8dee7; border-radius: 13px; padding: 10px; margin-bottom: 10px; }
  li.active { border-color: #7aa7ff; background: #f5f8ff; }
  .cover { display: flex; align-items: center; justify-content: center; flex-shrink: 0; width: 42px; height: 46px; border-radius: 10px; background: #f0f3f7; color: #7d8a9d; }
  h3 { font-size: .875rem; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; margin: 0 0 6px; }
  .track-info p { color: #637184; font-size: .75rem; margin: 0; }
  .play-button { width: 34px; height: 36px; background: transparent; color: #1599c8; }
  .play-button:hover { background: #edf8fc; }
  .empty-state { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; text-align: center; color: #637184; padding: 32px 12px; }
  .empty-state p { margin: 0; color: #34445a; font-size: .9rem; }
  .empty-state span { max-width: 230px; line-height: 1.6; font-size: .8rem; }
  .local-player { margin-top: auto; padding-top: 14px; border-top: 1px solid #edf0f5; }
  .local-player p { font-size: .75rem; color: #536170; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  audio { width: 100%; height: 36px; }
  @media (max-width: 720px) { .library { height: min(580px, 75dvh); min-height: 340px; } }
</style>
