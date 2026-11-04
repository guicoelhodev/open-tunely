import { invoke } from "@tauri-apps/api/core";
import { requireDesktop } from "./music.service";

export type LocalAudioSource = { url: string; dispose: () => void };

export async function loadLocalAudio(path: string, signal: AbortSignal): Promise<LocalAudioSource> {
  requireDesktop();
  signal.throwIfAborted();
  const url = await invoke<string>("local_audio_url", { path });
  let released = false;
  const source = {
    url,
    dispose() {
      if (released) return;
      released = true;
      void invoke("release_local_audio", { url }).catch(console.warn);
    },
  };
  if (signal.aborted) {
    source.dispose();
    signal.throwIfAborted();
  }
  return source;
}
