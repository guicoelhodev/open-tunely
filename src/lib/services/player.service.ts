import { writable, get } from "svelte/store";
import type { LocalAudioSource } from "./local-audio.service";
export type PlayerTrack = { id: string; title: string; subtitle?: string; load: (signal: AbortSignal) => Promise<LocalAudioSource> };
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
    const loaded = await track.load(controller.signal);
    if (generation !== current) { loaded.dispose(); return; }
    source = loaded;
    player.update(s => ({ ...s, url: loaded.url }));
    if (audio) { audio.src = loaded.url; await audio.play(); }
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
  if (!tracks.length || index < 0) return;
  if (ended && !state.shuffle && index === tracks.length - 1) return;
  const candidates = tracks.filter(t => t.id !== state.track?.id);
  const next = state.shuffle && candidates.length ? candidates[Math.floor(Math.random() * candidates.length)] : tracks[(index + direction + tracks.length) % tracks.length];
  void playTrack(next, queue);
}
