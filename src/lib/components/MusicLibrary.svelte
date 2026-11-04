<script lang="ts">
  import Icon from "@iconify/svelte";
  import folderIcon from "@iconify-icons/andrsrxn/folder-in";
  import playIcon from "@iconify-icons/andrsrxn/play";
  import pauseIcon from "@iconify-icons/andrsrxn/play-off";
  import musicIcon from "@iconify-icons/andrsrxn/music-note";
  import { convertFileSrc, invoke, isTauri } from "@tauri-apps/api/core";
  import { onDestroy, tick } from "svelte";

  type Track = { path: string; title: string; filename: string; size: number };
  type Library = { folder: string; tracks: Track[] };
  let { onplay }: { onplay: () => void } = $props();
  let library = $state<Library | null>(null);
  let choosing = $state(false);
  let error = $state("");
  let active = $state<Track | null>(null);
  let playing = $state(false);
  let audio = $state<HTMLAudioElement>();
  let request = 0;
  let destroyed = false;

  export function stop() {
    request += 1;
    audio?.pause();
    audio?.removeAttribute("src");
    audio?.load();
    active = null;
    playing = false;
  }

  onDestroy(() => {
    destroyed = true;
    stop();
  });

  async function chooseFolder() {
    if (choosing) return;
    if (!isTauri()) {
      error = "Abra a janela desktop do aplicativo para selecionar uma pasta.";
      return;
    }
    choosing = true;
    error = "";
    try {
      const selected = await invoke<Library | null>("choose_music_folder");
      if (destroyed || !selected) return;
      stop();
      library = selected;
    } catch (cause) {
      if (!destroyed) error = String(cause);
    } finally {
      if (!destroyed) choosing = false;
    }
  }

  async function toggle(track: Track) {
    if (active?.path === track.path && playing) {
      audio?.pause();
      return;
    }
    const current = ++request;
    error = "";
    onplay();
    if (active?.path !== track.path) {
      audio?.pause();
      playing = false;
      active = track;
      await tick();
    }
    if (current !== request || destroyed) return;
    try {
      // Reload a failed source without resetting normal pause/resume playback.
      if (audio?.error) audio.load();
      await audio?.play();
    } catch {
      if (current === request && !destroyed) {
        error = "Não foi possível reproduzir este MP3. Verifique se o arquivo está disponível e tente novamente.";
      }
    }
  }

  function started() {
    playing = true;
    onplay();
  }

  function failed() {
    playing = false;
    error = "Não foi possível abrir este MP3. Ele pode ter sido movido ou estar em um formato incompatível.";
  }

  function fileSize(bytes: number) {
    return `${(bytes / 1024 / 1024).toLocaleString("pt-BR", { maximumFractionDigits: 1 })} MB`;
  }
</script>

<aside aria-label="Biblioteca de músicas" class="library">
  <header>
    <div class="folder-info">
      <h2>Minha biblioteca</h2>
      <p class="folder-path" title={library?.folder}>
        {library?.folder ?? "Selecione sua pasta de músicas"}
      </p>
    </div>
    <button class="folder-button" type="button" onclick={chooseFolder} disabled={choosing}
      aria-label="Selecionar pasta de músicas" title="Selecionar pasta de músicas">
      <Icon icon={folderIcon} width="26" />
    </button>
  </header>

  <div class="library-status" aria-live="polite" aria-busy={choosing}>
    {#if choosing}<p>Carregando pasta…</p>{/if}
    {#if error}<p class="error" role="alert">{error}</p>{/if}
  </div>

  {#if library}
    <p class="track-count">{library.tracks.length} {library.tracks.length === 1 ? "música" : "músicas"}</p>
    {#if library.tracks.length}
      <ul>
        {#each library.tracks as track (track.path)}
          <li class:active={active?.path === track.path}>
            <div class="cover" aria-hidden="true"><Icon icon={musicIcon} width="24" /></div>
            <div class="track-info">
              <h3 title={track.filename}>{track.title}</h3>
              <p>MP3 <span aria-hidden="true">·</span> {fileSize(track.size)}</p>
            </div>
            <button class="play-button" type="button" onclick={() => toggle(track)}
              aria-label={`${active?.path === track.path && playing ? "Pausar" : "Ouvir"} ${track.title}`}
              title={active?.path === track.path && playing ? "Pausar" : "Ouvir"}>
              <Icon icon={active?.path === track.path && playing ? pauseIcon : playIcon} width="25" />
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
  {:else if !choosing}
    <div class="empty-state">
      <Icon icon={folderIcon} width="36" />
      <p>Suas músicas, por aqui.</p>
      <span>Selecione uma pasta para listar e ouvir seus arquivos MP3.</span>
    </div>
  {/if}

  {#if active}
    <div class="local-player">
      <p title={active.title}>Tocando: {active.title}</p>
      <audio bind:this={audio} src={convertFileSrc(active.path)} controls preload="metadata"
        aria-label={`Reproduzir ${active.title}`} onplay={started} onpause={() => playing = false}
        onended={() => playing = false} onerror={failed}>
        <track kind="captions" />
      </audio>
    </div>
  {/if}
</aside>

<style>
  .library { display: flex; flex-direction: column; min-width: 0; min-height: 440px; height: calc(100dvh - 64px); box-sizing: border-box; border: 1px solid #d8dee7; border-radius: 24px; background: #fff; padding: 20px 16px; }
  header { display: flex; align-items: center; gap: 12px; padding: 0 4px 18px; border-bottom: 1px solid #edf0f5; }
  .folder-info, .track-info { flex: 1; min-width: 0; }
  h2 { margin: 0 0 8px; font-size: 1rem; }
  .folder-path { margin: 0; font-family: ui-monospace, monospace; font-size: .8rem; line-height: 1.5; overflow-wrap: anywhere; color: #536170; max-height: 4.5em; overflow: auto; }
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
  .play-button { width: 34px; height: 36px; background: transparent; color: #17202a; }
  .empty-state { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; text-align: center; color: #637184; padding: 32px 12px; }
  .empty-state p { margin: 0; color: #34445a; font-size: .9rem; }
  .empty-state span { max-width: 230px; line-height: 1.6; font-size: .8rem; }
  .local-player { margin-top: auto; padding-top: 14px; border-top: 1px solid #edf0f5; }
  .local-player p { font-size: .75rem; color: #536170; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  audio { width: 100%; height: 36px; }
  @media (max-width: 720px) { .library { height: min(580px, 75dvh); min-height: 340px; } }
</style>
