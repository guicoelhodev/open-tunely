<script lang="ts">
  import DownloadQueue from "./DownloadQueue.svelte";
  import TrackActions from "./TrackActions.svelte";
  import Icon from "@iconify/svelte";
  import folderIcon from "@iconify-icons/andrsrxn/folder-in";
  import playIcon from "@iconify-icons/andrsrxn/play";
  import pauseIcon from "@iconify-icons/andrsrxn/pause";
  import musicIcon from "@iconify-icons/andrsrxn/music-note";
  import searchIcon from "@iconify-icons/andrsrxn/search";
  import { library, choosingFolder, chooseMusicFolder, initializeMusicLibrary, type Track } from "$lib/services/library.service";
  import { downloading, downloadNotice } from "$lib/services/download.service";
  import { loadLocalAudio } from "$lib/services/local-audio.service";
  import { player, playTrack, stopPlayer, togglePlayer } from "$lib/services/player.service";
  import { onMount, onDestroy, untrack } from "svelte";


  const folderLabel = $derived.by(() => {
    if (!$library) return "Selecione sua pasta de músicas";
    const parts = $library.folder.split(/[\\/]+/).filter(Boolean);
    return `${parts.length > 2 ? ".../" : ""}${parts.slice(-2).join("/")}`;
  });
  let error = $state("");
  let query = $state("");
  function normalizeSearch(value: string) {
    return value.normalize("NFD").replace(/[\u0300-\u036f]/g, "").toLocaleLowerCase("pt-BR").trim();
  }
  const searchTerm = $derived(normalizeSearch(query));
  const filteredTracks = $derived(($library?.tracks ?? []).filter(track =>
    normalizeSearch(`${track.title} ${track.filename}`).includes(searchTerm)
  ));
  let trackActions: TrackActions;
  const active = $derived($player.track);
  const playing = $derived($player.playing);
  const preparing = $derived($player.loading ? $player.track?.id : null);
  let destroyed = false;
  export function stop() { stopPlayer(); }
  onMount(() => { void initializeMusicLibrary().catch(cause => { if (!destroyed) error = String(cause); }); });
  onDestroy(() => { destroyed = true; });
  let previousFolder: string | undefined;
  $effect(() => {
    const folder = $library?.folder;
    if (folder !== previousFolder) { previousFolder = folder; query = ""; untrack(stopPlayer); }
  });
  async function chooseFolder() {
    error = "";
    try { await chooseMusicFolder(); } catch (cause) { error = String(cause); }
  }
  function libraryQueue() {
    return ($library?.tracks ?? []).map(item => ({ id: item.path, title: item.title, subtitle: "MP3 · Biblioteca", load: (signal: AbortSignal) => loadLocalAudio(item.path, signal) }));
  }
  function toggle(track: Track) {
    if ($player.track?.id === track.path) { void togglePlayer(); return; }
    const tracks = libraryQueue();
    const selected = tracks.find(item => item.id === track.path);
    if (selected) void playTrack(selected, libraryQueue);
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
    <div class="-mx-1 flex min-h-0 flex-1 flex-col overflow-y-auto px-1" onscroll={() => trackActions?.dismiss()}>
      <div class="sticky top-0 z-10 shrink-0 bg-white py-3">
        <label for="library-query" class="sr-only">Buscar na minha biblioteca</label>
        <div class="relative">
          <span class="pointer-events-none absolute inset-y-0 left-3 flex items-center text-[#637184]" aria-hidden="true"><Icon icon={searchIcon} width="18" /></span>
          <input id="library-query" type="search" bind:value={query} placeholder="Buscar na biblioteca…"
            class="w-full min-w-0 rounded-lg border border-[#8b949e] bg-white py-2.5 pr-3 pl-10 text-sm focus-visible:outline-3 focus-visible:outline-solid focus-visible:outline-[#7aa7ff] focus-visible:outline-offset-[3px]" />
        </div>
      </div>
      <p class="mx-1 mb-3 shrink-0 text-xs text-[#637184]" role="status">
        {#if searchTerm}{filteredTracks.length} de {/if}{$library.tracks.length} {$library.tracks.length === 1 ? "música" : "músicas"}
      </p>
      {#if filteredTracks.length}
        <ul class="shrink-0 list-none pt-0.5 pb-2">
          {#each filteredTracks as track (track.path)}
            <li class="mb-2.5 flex items-center gap-2.5 rounded-[13px] border p-2.5 {active?.id === track.path ? 'border-[#7aa7ff] bg-[#f5f8ff]' : 'border-[#d8dee7]'}" oncontextmenu={(event) => trackActions.open(event, track)}>
              <div class="flex h-[46px] w-[42px] shrink-0 items-center justify-center rounded-[10px] bg-[#f0f3f7] text-[#7d8a9d]" aria-hidden="true"><Icon icon={musicIcon} width="24" /></div>
              <div class="min-w-0 flex-1">
                <h3 class="mb-1.5 truncate text-sm font-semibold" title={track.filename}>{track.title}</h3>
                <p class="text-xs text-[#637184]">MP3 <span aria-hidden="true">·</span> {fileSize(track.size)}</p>
              </div>
              <button class="inline-flex h-9 w-[34px] shrink-0 cursor-pointer items-center justify-center rounded-[10px] bg-transparent p-0 text-[#1599c8] hover:bg-[#edf8fc] disabled:cursor-wait disabled:opacity-50 focus-visible:outline-3 focus-visible:outline-solid focus-visible:outline-[#7aa7ff] focus-visible:outline-offset-[3px]" type="button" onclick={() => toggle(track)}
                onkeydown={(event) => { if (event.key === "ContextMenu" || (event.shiftKey && event.key === "F10")) void trackActions.open(event, track); }}
                disabled={preparing === track.path}
                aria-label={`${active?.id === track.path && playing ? "Pausar" : "Ouvir"} ${track.title}`}
                title={active?.id === track.path && playing ? "Pausar" : "Ouvir"}>
                <Icon icon={active?.id === track.path && playing ? pauseIcon : playIcon} width="20" />
              </button>
            </li>
          {/each}
        </ul>
      {:else}
        <div class="flex flex-1 flex-col items-center justify-center gap-3 px-3 py-8 text-center text-[#637184]">
          <Icon icon={musicIcon} width="32" />
          <p class="text-[0.9rem] text-[#34445a]">{searchTerm ? "Nenhuma música encontrada." : "Nenhum arquivo MP3 nesta pasta."}</p>
          <span class="max-w-[230px] text-[0.8rem] leading-[1.6]">{searchTerm ? "Tente outro nome ou limpe a busca." : "Escolha outra pasta pelo ícone acima."}</span>
        </div>
      {/if}
    </div>
  {:else if !$choosingFolder}
    <div class="flex flex-1 flex-col items-center justify-center gap-3 px-3 py-8 text-center text-[#637184]">
      <Icon icon={folderIcon} width="36" />
      <p class="text-[0.9rem] text-[#34445a]">Suas músicas, por aqui.</p>
      <span class="max-w-[230px] text-[0.8rem] leading-[1.6]">Selecione uma pasta para listar e ouvir seus arquivos MP3.</span>
    </div>
  {/if}

  <DownloadQueue />
  <TrackActions bind:this={trackActions} onbeforechange={(track) => { if (active?.id === track.path) stop(); }} onnotice={(message) => error = message} />
</aside>
