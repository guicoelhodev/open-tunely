import { m } from "$lib/paraglide/messages.js";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { get, writable, readonly } from "svelte/store";
import { requireDesktop } from "./music.service";

export type Track = { path: string; title: string; filename: string; size: number };
export type Library = { folder: string; tracks: Track[] };

const currentLibrary = writable<Library | null>(null);
const selecting = writable(false);
export const library = readonly(currentLibrary);
export const choosingFolder = readonly(selecting);
let selection: Promise<Library | null> | undefined;
let revision = 0;
let initialization: Promise<void> | undefined;
const folderStorageKey = "tunely.musicFolder";

function savedFolder(): string | null {
  try { return localStorage.getItem(folderStorageKey); }
  catch { return null; }
}

function saveFolder(folder: string) {
  try { localStorage.setItem(folderStorageKey, folder); }
  catch { /* The current session still works when storage is unavailable. */ }
}

export function initializeMusicLibrary(): Promise<void> {
  if (!isTauri()) return Promise.resolve();
  return initialization ??= (async () => {
    const folder = savedFolder();
    if (!folder) return refreshMusicLibrary();
    const current = revision;
    const restored = await invoke<Library>("restore_music_folder", { path: folder });
    if (current === revision && !selection) {
      currentLibrary.set(restored);
      saveFolder(restored.folder);
    }
  })();
}

export function chooseMusicFolder(): Promise<Library | null> {
  requireDesktop();
  if (selection) return selection;
  revision += 1;
  selecting.set(true);
  selection = initializeMusicLibrary().catch(() => {}).then(() =>
    invoke<Library | null>("choose_music_folder", { title: m.text_selecionar_pasta_de_musicas() }),
  )
    .then((chosen) => {
      if (chosen) { currentLibrary.set(chosen); saveFolder(chosen.folder); }
      return chosen;
    })
    .finally(() => {
      selecting.set(false);
      selection = undefined;
    });
  return selection;
}

export async function ensureMusicFolder() {
  await initializeMusicLibrary().catch(() => {});
  // Wait for an open picker rather than downloading into its previous selection.
  return selection ?? Promise.resolve(get(currentLibrary) ?? null).then(
    (current) => current ?? chooseMusicFolder(),
  );
}

export async function refreshMusicLibrary() {
  const current = revision;
  const refreshed = await invoke<Library | null>("refresh_music_library");
  if (current === revision && !selection) currentLibrary.set(refreshed);
}

export async function renameMusicTrack(track: Track, name: string) {
  requireDesktop();
  await invoke("rename_music_track", { path: track.path, name });
}

export async function deleteMusicTrack(track: Track) {
  requireDesktop();
  await invoke("delete_music_track", { path: track.path });
}
