<script lang="ts">
  import DownloadQueue from "./DownloadQueue.svelte";
  import TrackActions from "./TrackActions.svelte";
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
  let trackActions: TrackActions;
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

<aside aria-label="Biblioteca de músicas" class="flex h-[calc(100dvh-64px)] min-h-[440px] min-w-0 flex-col rounded-3xl border border-[#d8dee7] bg-white px-4 py-5 max-[720px]:h-[min(580px,75dvh)] max-[720px]:min-h-[340px]">
  <header class="flex items-center gap-3 border-b border-[#edf0f5] px-1 pb-[18px]">
    <div class="min-w-0 flex-1">
      <h2 class="mb-2 text-base font-bold">Minha biblioteca</h2>
      <p class="truncate font-mono text-[0.8rem] leading-normal text-[#536170]" title={$library?.folder}>
        {folderLabel}
      </p>
    </div>
    <button class="inline-flex h-10 w-10 shrink-0 cursor-pointer items-center justify-center rounded-[10px] bg-transparent p-0 text-[#1599c8] hover:bg-[#edf8fc] disabled:cursor-wait disabled:opacity-50 focus-visible:outline-3 focus-visible:outline-solid focus-visible:outline-[#7aa7ff] focus-visible:outline-offset-[3px]" type="button" onclick={chooseFolder} disabled={$choosingFolder || $downloading}
      aria-label="Selecionar pasta de músicas" title="Selecionar pasta de músicas">
      <Icon icon={folderIcon} width="26" />
    </button>
  </header>

  <div class="[&_p]:my-4 [&_p]:text-[0.85rem] [&_p]:leading-normal" aria-live="polite" aria-busy={$choosingFolder || !!preparing}>
    {#if $choosingFolder}<p>Carregando pasta…</p>{/if}
    {#if error}<p class="text-[#a51e25]" role="alert">{error}</p>{/if}
    {#if $downloadNotice}<p class="text-[#a51e25]" role="alert">{$downloadNotice}</p>{/if}
  </div>

  {#if $library}
    <p class="mx-1 mt-[18px] mb-3 text-xs text-[#637184]">{$library.tracks.length} {$library.tracks.length === 1 ? "música" : "músicas"}</p>
    {#if $library.tracks.length}
      <ul class="-mx-1 min-h-0 flex-1 list-none overflow-y-auto px-1 pt-0.5 pb-2" onscroll={() => trackActions?.dismiss()}>
        {#each $library.tracks as track (track.path)}
          <li class="mb-2.5 flex items-center gap-2.5 rounded-[13px] border p-2.5 {active?.path === track.path ? 'border-[#7aa7ff] bg-[#f5f8ff]' : 'border-[#d8dee7]'}" oncontextmenu={(event) => trackActions.open(event, track)}>
            <div class="flex h-[46px] w-[42px] shrink-0 items-center justify-center rounded-[10px] bg-[#f0f3f7] text-[#7d8a9d]" aria-hidden="true"><Icon icon={musicIcon} width="24" /></div>
            <div class="min-w-0 flex-1">
              <h3 class="mb-1.5 truncate text-sm font-semibold" title={track.filename}>{track.title}</h3>
              <p class="text-xs text-[#637184]">MP3 <span aria-hidden="true">·</span> {fileSize(track.size)}</p>
            </div>
            <button class="inline-flex h-9 w-[34px] shrink-0 cursor-pointer items-center justify-center rounded-[10px] bg-transparent p-0 text-[#1599c8] hover:bg-[#edf8fc] disabled:cursor-wait disabled:opacity-50 focus-visible:outline-3 focus-visible:outline-solid focus-visible:outline-[#7aa7ff] focus-visible:outline-offset-[3px]" type="button" onclick={() => toggle(track)}
              onkeydown={(event) => { if (event.key === "ContextMenu" || (event.shiftKey && event.key === "F10")) void trackActions.open(event, track); }}
              disabled={preparing === track.path}
              aria-label={`${active?.path === track.path && playing ? "Pausar" : "Ouvir"} ${track.title}`}
              title={active?.path === track.path && playing ? "Pausar" : "Ouvir"}>
              <Icon icon={active?.path === track.path && playing ? pauseIcon : playIcon} width="20" />
            </button>
          </li>
        {/each}
      </ul>
    {:else}
      <div class="flex flex-1 flex-col items-center justify-center gap-3 px-3 py-8 text-center text-[#637184]">
        <Icon icon={musicIcon} width="32" />
        <p class="text-[0.9rem] text-[#34445a]">Nenhum arquivo MP3 nesta pasta.</p>
        <span class="max-w-[230px] text-[0.8rem] leading-[1.6]">Escolha outra pasta pelo ícone acima.</span>
      </div>
    {/if}
  {:else if !$choosingFolder}
    <div class="flex flex-1 flex-col items-center justify-center gap-3 px-3 py-8 text-center text-[#637184]">
      <Icon icon={folderIcon} width="36" />
      <p class="text-[0.9rem] text-[#34445a]">Suas músicas, por aqui.</p>
      <span class="max-w-[230px] text-[0.8rem] leading-[1.6]">Selecione uma pasta para listar e ouvir seus arquivos MP3.</span>
    </div>
  {/if}

  {#if active && playbackUrl}
    <div class="mt-auto border-t border-[#edf0f5] pt-3.5">
      <p class="my-3 truncate text-xs text-[#536170]" title={active.title}>Tocando: {active.title}</p>
      {#key playbackUrl}
        <audio class="h-9 w-full" bind:this={audio} src={playbackUrl} controls preload="metadata"
          aria-label={`Reproduzir ${active.title}`} onplay={started} onpause={paused}
          onended={paused} onerror={failed}>
          <track kind="captions" />
        </audio>
      {/key}
    </div>
  {/if}
  <DownloadQueue />
  <TrackActions bind:this={trackActions} onbeforechange={(track) => { if (active?.path === track.path) stop(); }} onnotice={(message) => error = message} />
</aside>
