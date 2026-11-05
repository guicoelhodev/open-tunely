<script lang="ts">
  import { errorMessage } from "$lib/services/errors.service";
  import { locale } from "$lib/services/locale.service";
  import { m } from "$lib/paraglide/messages.js";
  import Button from "./ui/Button.svelte";
  import DownloadQueue from "./DownloadQueue.svelte";
  import TrackActions from "./TrackActions.svelte";
  import Modal from "./ui/Modal.svelte";
  import Popover from "./ui/Popover.svelte";
  import Icon from "@iconify/svelte";
  import folderIcon from "@iconify-icons/andrsrxn/folder-in";
  import playIcon from "@iconify-icons/andrsrxn/play";
  import pauseIcon from "@iconify-icons/andrsrxn/pause";
  import musicIcon from "@iconify-icons/andrsrxn/music-note";
  import searchIcon from "@iconify-icons/andrsrxn/search";
  import languageIcon from "@iconify-icons/andrsrxn/translate";
  import themeIcon from "@iconify-icons/andrsrxn/moon";
  import sunIcon from "@iconify-icons/andrsrxn/sun";
  import settingsIcon from "@iconify-icons/andrsrxn/settings";
  import closeIcon from "@iconify-icons/andrsrxn/x";
  import checkIcon from "@iconify-icons/andrsrxn/check";
  import { library, choosingFolder, chooseMusicFolder, initializeMusicLibrary, type Track } from "$lib/services/library.service";
  import { downloading, downloadNotice } from "$lib/services/download.service";
  import { loadLocalAudio } from "$lib/services/local-audio.service";
  import { player, playTrack, stopPlayer, togglePlayer } from "$lib/services/player.service";
  import { appearance, palettes, selectPalette, toggleColorMode } from "$lib/services/appearance.service";
  import { onMount, onDestroy, untrack } from "svelte";


  const folderLabel = $derived.by(() => {
    if (!$library) return m.text_select_your_music_folder({}, { locale: $locale });
    const parts = $library.folder.split(/[\\/]+/).filter(Boolean);
    return `${parts.length > 2 ? ".../" : ""}${parts.slice(-2).join("/")}`;
  });
  let error = $state("");
  let query = $state("");
  let settingsTrigger: HTMLButtonElement | undefined = $state();
  let settingsDialog: HTMLDialogElement | undefined = $state();
  let languageTrigger: HTMLButtonElement | undefined = $state();
  let languagePanel: HTMLDivElement | undefined = $state();
  let themeTrigger: HTMLButtonElement | undefined = $state();
  let themePanel: HTMLDivElement | undefined = $state();
  import { getLocale, type Locale } from "$lib/paraglide/runtime.js";
  import { selectLocale } from "$lib/services/locale.service";
  const languages: { id: Locale; label: string; flag: string }[] = [
    { id: "pt-BR", label: "Brasil", flag: "🇧🇷" },
    { id: "en", label: "English", flag: "🇬🇧" },
    { id: "es", label: "Español", flag: "🇪🇸" },
  ];
  function positionOptions(event: ToggleEvent, trigger?: HTMLButtonElement, panel?: HTMLDivElement) {
    if (event.newState !== "open" || !trigger || !panel) return;
    const bounds = trigger.getBoundingClientRect();
    const height = panel.offsetHeight;
    panel.style.right = `${Math.max(8, window.innerWidth - bounds.right)}px`;
    panel.style.top = `${Math.max(8, bounds.bottom + height + 8 <= window.innerHeight - 8 ? bounds.bottom + 8 : bounds.top - height - 8)}px`;
  }
  function selectLanguage(id: Locale) {
    selectLocale(id);
    closePreferencePopovers();
    languageTrigger?.focus();
  }
  function closePreferencePopovers() {
    languagePanel?.hidePopover();
    themePanel?.hidePopover();
  }
  function normalizeSearch(value: string) {
    return value.normalize("NFD").replace(/[\u0300-\u036f]/g, "").toLocaleLowerCase(getLocale()).trim();
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
    return filteredTracks.map(item => ({ id: item.path, title: item.title, subtitle: m.text_mp3_library({}, { locale: $locale }), load: (signal: AbortSignal) => loadLocalAudio(item.path, signal) }));
  }
  function toggle(track: Track) {
    if ($player.track?.id === track.path) { void togglePlayer(); return; }
    const tracks = libraryQueue();
    const selected = tracks.find(item => item.id === track.path);
    if (selected) void playTrack(selected, libraryQueue);
  }

  function fileSize(bytes: number) {
    return `${(bytes / 1024 / 1024).toLocaleString(getLocale(), { maximumFractionDigits: 1 })} MB`;
  }
</script>

<svelte:window onresize={closePreferencePopovers} />

<aside aria-label={m.text_music_library({}, { locale: $locale })} class="flex h-[calc(100dvh-64px)] min-h-[440px] min-w-0 flex-col rounded-xl border border-border bg-card px-4 py-5 pr-0 max-[720px]:h-full max-[720px]:min-h-0 max-[720px]:overflow-hidden">
  <header class="flex items-center gap-3 border-b border-border px-1 pb-[18px]">
    <div class="min-w-0 flex-1">
      <h2 class="mb-2 text-base font-bold">{m.text_my_library({}, { locale: $locale })}</h2>
      <p class="truncate font-mono text-[0.8rem] leading-normal text-muted-foreground" title={$library?.folder}>
        {folderLabel}
      </p>
    </div>
    <Button variant="ghost" size="custom" class="h-10 w-10 shrink-0 p-0" type="button" onclick={chooseFolder} disabled={$choosingFolder || $downloading}
      aria-label={m.text_select_music_folder({}, { locale: $locale })} title={m.text_select_music_folder({}, { locale: $locale })}>
      <Icon icon={folderIcon} width="26" />
    </Button>
  </header>

  <div class="[&_p]:my-4 [&_p]:text-[0.85rem] [&_p]:leading-normal" aria-live="polite" aria-busy={$choosingFolder || !!preparing}>
    {#if $choosingFolder}<p>{m.text_loading_folder({}, { locale: $locale })}</p>{/if}
    {#if error}<p class="text-red-700 dark:text-red-300" role="alert">{errorMessage(error, $locale)}</p>{/if}
    {#if $downloadNotice}<p class="text-red-700 dark:text-red-300" role="alert">{$downloadNotice}</p>{/if}
  </div>

  {#if $library}
    <div class="-mx-1 flex min-h-0 flex-1 flex-col overflow-y-auto px-1" onscroll={() => trackActions?.dismiss()}>
      <div class="sticky top-0 z-10 shrink-0 bg-card py-3 pr-4">
        <label for="library-query" class="sr-only">{m.text_search_my_library({}, { locale: $locale })}</label>
        <div class="relative">
          <span class="pointer-events-none absolute inset-y-0 left-3 flex items-center text-muted-foreground" aria-hidden="true"><Icon icon={searchIcon} width="18" /></span>
          <input id="library-query" type="search" bind:value={query} placeholder={m.text_search_library({}, { locale: $locale })}
            class="w-full min-w-0 rounded-md border border-input bg-card py-2.5 pr-3 pl-10 text-sm focus-visible:outline-3 focus-visible:outline-solid focus-visible:outline-ring focus-visible:outline-offset-[3px]" />
        </div>
      </div>
      <p class="mx-1 mb-3 shrink-0 text-xs text-muted-foreground" role="status">
        {#if searchTerm}{m.count_of({ filtered: filteredTracks.length, total: $library.tracks.length }, { locale: $locale })}{:else}{$library.tracks.length}{/if} {$library.tracks.length === 1 ? m.text_song({}, { locale: $locale }) : m.text_songs({}, { locale: $locale })}
      </p>
      {#if filteredTracks.length}
        <ul class="shrink-0 list-none pt-0.5 pr-4 pb-2">
          {#each filteredTracks as track (track.path)}
            <li class="mb-2.5 flex items-center gap-2.5 rounded-lg border p-2.5 {active?.id === track.path ? 'border-primary bg-primary/5' : 'border-border'}" oncontextmenu={(event) => trackActions.open(event, track)}>
              <div class="flex h-[46px] w-[42px] shrink-0 items-center justify-center rounded-md bg-primary/7 text-primary" aria-hidden="true"><Icon icon={musicIcon} width="24" /></div>
              <div class="min-w-0 flex-1">
                <h3 class="mb-1.5 truncate text-sm font-semibold" title={track.filename}>{track.title}</h3>
                <p class="text-xs text-muted-foreground">MP3 <span aria-hidden="true">·</span> {fileSize(track.size)}</p>
              </div>
              <Button variant="ghost" size="custom" class="h-9 w-[34px] shrink-0 p-0" type="button" onclick={() => toggle(track)}
                onkeydown={(event) => { if (event.key === "ContextMenu" || (event.shiftKey && event.key === "F10")) void trackActions.open(event, track); }}
                disabled={preparing === track.path}
                aria-label={m.action_track({ action: active?.id === track.path && playing ? m.text_pause({}, { locale: $locale }) : m.text_listen({}, { locale: $locale }), title: track.title }, { locale: $locale })}
                title={active?.id === track.path && playing ? m.text_pause({}, { locale: $locale }) : m.text_listen({}, { locale: $locale })}>
                <Icon icon={active?.id === track.path && playing ? pauseIcon : playIcon} width="20" />
              </Button>
            </li>
          {/each}
        </ul>
      {:else}
        <div class="flex flex-1 flex-col items-center justify-center gap-3 px-3 py-8 text-center text-muted-foreground">
          <Icon icon={musicIcon} width="32" />
          <p class="text-[0.9rem] text-foreground">{searchTerm ? m.text_no_songs_found({}, { locale: $locale }) : m.text_no_mp3_files_in_this_folder({}, { locale: $locale })}</p>
          <span class="max-w-[230px] text-[0.8rem] leading-[1.6]">{searchTerm ? m.text_try_another_name_or_clear_the_search({}, { locale: $locale }) : m.text_choose_another_folder_using_the_icon_above({}, { locale: $locale })}</span>
        </div>
      {/if}
    </div>
  {:else if !$choosingFolder}
    <div class="flex flex-1 flex-col items-center justify-center gap-3 px-3 py-8 text-center text-muted-foreground">
      <Icon icon={folderIcon} width="36" />
      <p class="text-[0.9rem] text-foreground">{m.text_your_music_right_here({}, { locale: $locale })}</p>
      <span class="max-w-[230px] text-[0.8rem] leading-[1.6]">{m.text_select_a_folder_to_list_and_listen_to_your_mp3_files({}, { locale: $locale })}</span>
    </div>
  {/if}

  <DownloadQueue />
  <TrackActions bind:this={trackActions} onbeforechange={(track) => { if (active?.id === track.path) stop(); }} onnotice={(message) => error = message} />

  <footer aria-label={m.text_about_and_app_preferences({}, { locale: $locale })} class="mr-4 mt-5 flex shrink-0 items-center justify-between gap-3 border-t border-border px-1 pt-5">
    <div class="min-w-0">
      <p class="truncate text-base font-bold text-foreground">Open Tunely</p>
      <p class="mt-0.5 text-xs text-muted-foreground">v0.0.1</p>
    </div>
    <div class="flex shrink-0 items-center gap-1.5" role="group" aria-label={m.text_app_preferences({}, { locale: $locale })}>
      <Button type="button" onclick={toggleColorMode} aria-label={m.text_dark_mode({}, { locale: $locale })} aria-pressed={$appearance.mode === "dark"} title={$appearance.mode === "dark" ? m.text_enable_light_mode({}, { locale: $locale }) : m.text_enable_dark_mode({}, { locale: $locale })}
        variant="subtle" size="icon">
        <Icon icon={$appearance.mode === "dark" ? sunIcon : themeIcon} width="19" />
      </Button>
      <Button type="button" bind:element={settingsTrigger} onclick={() => settingsDialog?.showModal()} aria-haspopup="dialog" aria-controls="app-settings" aria-label={m.text_settings({}, { locale: $locale })} title={m.text_settings({}, { locale: $locale })}
        variant="subtle" size="icon">
        <Icon icon={settingsIcon} width="19" />
      </Button>
    </div>
  </footer>
  <Modal id="app-settings" bind:element={settingsDialog} size="wide" onscroll={closePreferencePopovers} onclose={() => { closePreferencePopovers(); settingsTrigger?.focus(); }} aria-labelledby="app-settings-title"
    class="max-h-[calc(100dvh-48px)] overflow-y-auto shadow-[0_8px_28px_#17202a26]">
    <header class="mb-6 flex items-center justify-between gap-4">
      <h3 id="app-settings-title" class="text-lg font-bold">{m.text_settings({}, { locale: $locale })}</h3>
      <Button type="button" onclick={() => settingsDialog?.close()} aria-label={m.text_close_settings({}, { locale: $locale })} title={m.text_close_settings({}, { locale: $locale })}
        variant="muted" size="icon" class="shrink-0">
        <Icon icon={closeIcon} width="20" />
      </Button>
    </header>
    <div class="flex items-center justify-between gap-4">
      <div>
        <p class="text-sm text-foreground">{m.text_color_theme({}, { locale: $locale })}</p>
        <p class="mt-0.5 text-xs text-muted-foreground">{palettes.find(palette => palette.id === $appearance.palette)?.label($locale)}</p>
      </div>
      <Button type="button" bind:element={themeTrigger} popovertarget="theme-options" aria-label={m.text_select_color_theme({}, { locale: $locale })} title={m.text_select_color_theme({}, { locale: $locale })}
        variant="subtle" size="icon" class="shrink-0">
        <span class="size-[18px] rounded-full bg-primary" aria-hidden="true"></span>
      </Button>
    </div>
    <div class="mt-3 flex items-center justify-between gap-4 border-t border-border pt-3">
      <div>
        <p class="text-sm text-foreground">{m.text_language({}, { locale: $locale })}</p>
        <p class="mt-0.5 text-xs text-muted-foreground">{languages.find(language => language.id === $locale)?.label}</p>
      </div>
      <Button type="button" bind:element={languageTrigger} popovertarget="language-options" aria-label={m.text_select_language({}, { locale: $locale })} title={m.text_select_language({}, { locale: $locale })}
        variant="subtle" size="icon" class="shrink-0">
        <Icon icon={languageIcon} width="19" />
      </Button>
    </div>
    <Popover id="theme-options" bind:element={themePanel} ontoggle={(event) => positionOptions(event, themeTrigger, themePanel)} role="group" aria-label={m.text_available_color_themes({}, { locale: $locale })}
      class="inset-auto w-[min(200px,calc(100vw-32px))] rounded-lg p-1.5 shadow-[0_8px_28px_#17202a26]">
      {#each palettes as palette}
        <Button type="button" aria-pressed={$appearance.palette === palette.id} onclick={() => { selectPalette(palette.id); closePreferencePopovers(); themeTrigger?.focus(); }}
          variant="option" size="md" layout="menu" class="w-full">
          <span class="size-[18px] shrink-0 rounded-full" style:background-color={$appearance.mode === "dark" ? palette.darkColor : palette.color} aria-hidden="true"></span>
          <span class="flex-1">{palette.label($locale)}</span>
          {#if $appearance.palette === palette.id}<Icon icon={checkIcon} width="17" />{/if}
        </Button>
      {/each}
    </Popover>
    <Popover id="language-options" bind:element={languagePanel} ontoggle={(event) => positionOptions(event, languageTrigger, languagePanel)} role="group" aria-label={m.text_available_languages({}, { locale: $locale })}
      class="inset-auto w-[min(200px,calc(100vw-32px))] rounded-lg p-1.5 shadow-[0_8px_28px_#17202a26]">
      {#each languages as language}
        <Button type="button" aria-pressed={$locale === language.id} onclick={() => selectLanguage(language.id)}
          variant="option" size="md" layout="menu" class="w-full">
          <span class="text-xl leading-none" aria-hidden="true">{language.flag}</span>
          <span class="flex-1">{language.label}</span>
          {#if $locale === language.id}<Icon icon={checkIcon} width="17" />{/if}
        </Button>
      {/each}
    </Popover>
  </Modal>
</aside>
