import { invoke } from "@tauri-apps/api/core";
import { readonly, writable } from "svelte/store";
import { ensureMusicFolder, refreshMusicLibrary } from "./library.service";
import { requireDesktop } from "./music.service";

const busy = writable(false);
export const downloading = readonly(busy);
let pending: Promise<DownloadResult | null> | undefined;
export type DownloadResult = { path: string; libraryWarning?: string };

export function downloadSong(id: string): Promise<DownloadResult | null> {
  requireDesktop();
  if (pending) return Promise.reject(new Error("Aguarde o download atual terminar."));
  busy.set(true);
  pending = (async () => {
    const folder = await ensureMusicFolder();
    if (!folder) return null;
    const path = await invoke<string>("download_song", { id });
    try {
      await refreshMusicLibrary();
      return { path };
    } catch {
      return { path, libraryWarning: "O MP3 foi salvo, mas não foi possível atualizar a biblioteca. Selecione a pasta novamente." };
    }
  })().finally(() => {
    busy.set(false);
    pending = undefined;
  });
  return pending;
}
