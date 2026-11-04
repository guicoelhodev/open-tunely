import { convertFileSrc } from "@tauri-apps/api/core";
import { requireDesktop } from "./music.service";

export type LocalAudioSource = { url: string; dispose: () => void };

export async function loadLocalAudio(path: string, signal: AbortSignal): Promise<LocalAudioSource> {
  requireDesktop();
  // WebKitGTK's media loader can reject asset:// even when fetch can read it.
  // Keep Tauri's per-file scope, then give the player a standard Blob URL.
  let data: Blob;
  try {
    const response = await fetch(convertFileSrc(path), { signal });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    data = await response.blob();
  } catch {
    signal.throwIfAborted();
    throw new Error("Não foi possível ler este MP3. Selecione a pasta novamente e verifique as permissões do arquivo.");
  }
  signal.throwIfAborted();
  const url = URL.createObjectURL(data.type === "audio/mpeg" ? data : new Blob([data], { type: "audio/mpeg" }));
  let released = false;
  return {
    url,
    dispose() {
      if (released) return;
      released = true;
      URL.revokeObjectURL(url);
    },
  };
}
