import { writable, get } from "svelte/store";
import type { LocalAudioSource } from "./local-audio.service";
export type PlayerTrack = { id: string; title: string; subtitle?: string; refreshOnPlaybackError?: boolean; load: (signal: AbortSignal) => Promise<LocalAudioSource> };
export const player = writable({ track: null as PlayerTrack | null, url: "", playing: false, loading: false, error: "", shuffle: false });
type PlayerQueue = PlayerTrack[] | (() => PlayerTrack[]);
let queue: PlayerQueue = [];
function currentQueue() { return typeof queue === "function" ? queue() : queue; }
let controller: AbortController | undefined;
let source: LocalAudioSource | undefined;
let audio: HTMLAudioElement | undefined;
let generation = 0;
export function attachPlayer(element: HTMLAudioElement) { audio = element; return () => { stopPlayer(); audio = undefined; }; }
export function stopPlayer() {
  generation++; controller?.abort(); audio?.pause(); audio?.removeAttribute("src"); audio?.load(); source?.dispose(); source = undefined;
  player.update(s => ({ ...s, track: null, url: "", playing: false, loading: false, error: "" }));
}
export async function playTrack(track: PlayerTrack, tracks: PlayerQueue = [track]) {
  stopPlayer(); queue = tracks;
  const current = generation;
  controller = new AbortController();
  player.update(s => ({ ...s, track, loading: true }));
  try {
    for (let attempt = 0; ; attempt++) {
      const loaded = await track.load(controller.signal);
      if (generation !== current) { loaded.dispose(); return; }
      source = loaded;
      player.update(s => ({ ...s, url: loaded.url, error: "" }));
      try {
        if (audio) { audio.src = loaded.url; await audio.play(); }
        break;
      } catch (error) {
        // Refresh a rejected temporary stream once. Permission/autoplay errors
        // and failures to resolve the source are not retried.
        if (generation !== current || attempt !== 0 || !track.refreshOnPlaybackError ||
            !audio?.error || ![2, 3, 4].includes(audio.error.code)) throw error;
        loaded.dispose(); source = undefined;
        audio.pause(); audio.removeAttribute("src"); audio.load();
        player.update(s => ({ ...s, url: "", error: "" }));
      }
    }
  } catch (error) {
    if (generation === current) player.update(s => ({ ...s, error: String(error) }));
  } finally {
    if (generation === current) player.update(s => ({ ...s, loading: false }));
  }
}
export async function togglePlayer() {
  const state = get(player);
  if (!audio || !state.track || state.loading) return;
  if (!state.url || audio.error) {
    await playTrack(state.track, queue);
    return;
  }
  if (!audio.paused) audio.pause();
  else { try { await audio.play(); } catch (error) { player.update(s => ({ ...s, error: String(error) })); } }
}
export function stepPlayer(direction: number, ended = false) {
  const state = get(player);
  const tracks = currentQueue();
  const index = tracks.findIndex(t => t.id === state.track?.id);
  if (!state.track || !tracks.length) return;
  if (ended && !state.shuffle && index === tracks.length - 1) return;
  const candidates = tracks.filter(t => t.id !== state.track?.id);
  // A live filter can exclude the playing track without interrupting its audio.
  const nextIndex = index < 0 ? (direction < 0 ? tracks.length - 1 : 0) : (index + direction + tracks.length) % tracks.length;
  const next = state.shuffle && candidates.length ? candidates[Math.floor(Math.random() * candidates.length)] : tracks[nextIndex];
  void playTrack(next, queue);
}
