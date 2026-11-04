import { Channel, invoke } from "@tauri-apps/api/core";
import { derived, get, readonly, writable } from "svelte/store";
import { ensureMusicFolder, refreshMusicLibrary } from "./library.service";
import { requireDesktop, type Song } from "./music.service";

export type DownloadStatus = "queued" | "downloading" | "converting" | "completed" | "error";
export type DownloadItem = Song & {
  key: number;
  status: DownloadStatus;
  percent: number | null;
  error?: string;
};
type Progress = { percent: number | null; converting: boolean };
const items = writable<DownloadItem[]>([]);
const notice = writable("");
export const downloadNotice = readonly(notice);
export const downloadQueue = readonly(items);
export const downloading = derived(items, ($items) =>
  $items.some((item) => ["queued", "downloading", "converting"].includes(item.status)),
);
let sequence = 0;
let running = false;

function update(key: number, patch: Partial<DownloadItem>) {
  items.update((queue) => queue.map((item) => item.key === key ? { ...item, ...patch } : item));
}

export async function downloadSong(song: Song): Promise<void> {
  requireDesktop();
  if (get(items).some((item) => item.id === song.id && item.status !== "completed")) return;
  if (!await ensureMusicFolder()) return;
  // The folder picker is shared, so check again after awaiting it.
  if (get(items).some((item) => item.id === song.id && item.status !== "completed")) return;
  items.update((queue) => [...queue, { ...song, key: ++sequence, status: "queued", percent: 0 }]);
  void processQueue();
}

async function processQueue() {
  if (running) return;
  running = true;
  try {
    for (;;) {
      const item = get(items).find((entry) => entry.status === "queued");
      if (!item) break;
      update(item.key, { status: "downloading", percent: null, error: undefined });
      const progress = new Channel<Progress>();
      let acceptingProgress = true;
      progress.onmessage = (value) => {
        if (acceptingProgress) update(item.key, {
          percent: value.percent,
          status: value.converting ? "converting" : "downloading",
        });
      };
      try {
        await invoke<string>("download_song", { id: item.id, onProgress: progress });
        acceptingProgress = false;
        try {
          await refreshMusicLibrary();
          notice.set("");
        } catch {
          notice.set("MP3 salvo. Selecione a pasta novamente para atualizar a biblioteca.");
        }
        update(item.key, { status: "completed", percent: 100 });
      } catch (cause) {
        acceptingProgress = false;
        update(item.key, { status: "error", error: String(cause) });
      }
    }
  } finally {
    running = false;
    // Keep failures actionable after the batch ends.
    items.update((queue) => queue.filter((item) => item.status !== "completed"));
  }
}

export function retryDownload(key: number) {
  const item = get(items).find((entry) => entry.key === key);
  if (item?.status !== "error") return;
  update(key, { status: "queued", percent: 0, error: undefined });
  void processQueue();
}

export function removeDownload(key: number) {
  items.update((queue) => queue.filter((item) => item.key !== key || !["error", "completed"].includes(item.status)));
}
