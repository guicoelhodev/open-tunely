<script lang="ts">
  import { errorMessage } from "$lib/services/errors.service";
  import { locale } from "$lib/services/locale.service";
  import { m } from "$lib/paraglide/messages.js";
  import Button from "./ui/Button.svelte";
  import { onMount } from "svelte";
  import Icon from "@iconify/svelte";
  import playIcon from "@iconify-icons/andrsrxn/play";
  import pauseIcon from "@iconify-icons/andrsrxn/pause";
  import { player, attachPlayer, togglePlayer, stepPlayer } from "$lib/services/player.service";
  let audio: HTMLAudioElement;
  import previousIcon from "@iconify-icons/andrsrxn/skip-backward";
  import nextIcon from "@iconify-icons/andrsrxn/skip-forward";
  import shuffleIcon from "@iconify-icons/andrsrxn/shuffle";
  import volumeIcon from "@iconify-icons/andrsrxn/volume-high";
  let volume = $state(1);
  let currentTime = $state(0);
  let duration = $state(0);
  const seekableDuration = $derived(Number.isFinite(duration) && duration > 0 ? duration : 0);
  function formatTime(seconds: number) {
    const value = Number.isFinite(seconds) ? Math.max(0, Math.floor(seconds)) : 0;
    return `${Math.floor(value / 60)}:${String(value % 60).padStart(2, "0")}`;
  }
  function seek(event: Event) {
    if (!seekableDuration) return;
    audio.currentTime = Math.min(seekableDuration, Math.max(0, Number((event.currentTarget as HTMLInputElement).value)));
    currentTime = audio.currentTime;
  }
  onMount(() => attachPlayer(audio));
  function playing(value: boolean) { player.update(s => ({ ...s, playing: value })); }
</script>

<footer class="mt-auto rounded-lg border border-border bg-card px-4 py-4" aria-label={m.text_music_player({}, { locale: $locale })}>
  <div class="flex flex-wrap items-center gap-4">
    <div class="min-w-0 flex-1"><p title={$player.track?.title} class="truncate text-sm font-semibold">{$player.track?.title ?? m.text_select_a_song({}, { locale: $locale })}</p>
      {#if $player.track?.subtitle}<p class="mt-1 truncate text-xs text-muted-foreground" title={$player.track.subtitle}>{$player.track.subtitle}</p>{/if}
      {#if $player.error}<p class="text-xs text-red-700 dark:text-red-300" role="alert">{errorMessage($player.error, $locale)}</p>{/if}
    </div>
    <div class="flex items-center gap-3 text-primary">
      <Button variant="ghost" size="custom" class="p-2" aria-label={m.text_previous({}, { locale: $locale })} disabled={!$player.track || $player.loading} onclick={() => stepPlayer(-1)}><Icon icon={previousIcon} width="20" /></Button>
      <Button variant="soft" size="custom" round class="p-3" aria-label={$player.playing ? m.text_pause({}, { locale: $locale }) : m.text_play({}, { locale: $locale })} disabled={!$player.track || $player.loading} onclick={togglePlayer}>
        {#if $player.loading}<span class="block size-5 animate-spin rounded-full border-2 border-current border-t-transparent" aria-hidden="true"></span>{:else}<Icon icon={$player.playing ? pauseIcon : playIcon} width="20" />{/if}
      </Button>
      <Button variant="ghost" size="custom" class="p-2" aria-label={m.text_next({}, { locale: $locale })} disabled={!$player.track || $player.loading} onclick={() => stepPlayer(1)}><Icon icon={nextIcon} width="20" /></Button>
      <Button variant={$player.shuffle ? "soft" : "muted"} size="custom" class="p-2" aria-label={m.text_shuffle({}, { locale: $locale })} aria-pressed={$player.shuffle} onclick={() => player.update(s => ({ ...s, shuffle: !s.shuffle }))}><Icon icon={shuffleIcon} width="20" /></Button>
    </div>
    <label class="flex items-center gap-2 text-muted-foreground"><Icon icon={volumeIcon} width="20" /><span class="sr-only">{m.text_volume({}, { locale: $locale })}</span><input class="w-24 accent-primary" type="range" min="0" max="1" step="0.01" bind:value={volume} /></label>
  </div>
  <div class="mt-3 flex items-center gap-3 text-xs tabular-nums text-muted-foreground">
    <span>{formatTime(currentTime)}</span>
    <input class="min-w-0 flex-1 cursor-pointer accent-primary disabled:cursor-default disabled:opacity-40" type="range" min="0" max={seekableDuration || 1} step="0.1" value={currentTime} disabled={!seekableDuration || $player.loading} oninput={seek} aria-label={m.text_song_position({}, { locale: $locale })} aria-valuetext={m.position_of({ current: formatTime(currentTime), total: formatTime(seekableDuration) }, { locale: $locale })} />
    <span>{formatTime(seekableDuration)}</span>
  </div>
</footer>
<audio bind:this={audio} bind:currentTime bind:duration bind:volume preload="metadata" onemptied={() => { currentTime = 0; duration = 0; }} onplay={() => playing(true)} onpause={() => playing(false)} onended={() => { playing(false); stepPlayer(1, true); }} onerror={() => player.update(s => ({ ...s, playing: false, error: m.text_could_not_play_this_audio({}, { locale: $locale }) }))}><track kind="captions" /></audio>
