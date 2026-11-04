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

export function initializeMusicLibrary(): Promise<void> {
  if (!isTauri()) return Promise.resolve();
  return initialization ??= refreshMusicLibrary();
}

export function chooseMusicFolder(): Promise<Library | null> {
  requireDesktop();
  if (selection) return selection;
  revision += 1;
  selecting.set(true);
  selection = invoke<Library | null>("choose_music_folder")
    .then((chosen) => {
      if (chosen) currentLibrary.set(chosen);
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
