import { invoke, isTauri } from "@tauri-apps/api/core";

export type Song = { id: string; title: string; channel?: string | null };

export function requireDesktop() {
  if (!isTauri()) throw new Error("Abra a janela desktop do aplicativo para usar esta função.");
}

export function searchSong(query: string) {
  requireDesktop();
  return invoke<Song>("search_song", { query });
}

export function reservePreview() {
  requireDesktop();
  return invoke<number>("reserve_preview");
}

export function resolvePreview(id: string, ticket: number) {
  return invoke<string>("preview_song", { id, ticket });
}

export function cancelPreview(ticket: number) {
  // A newer preview may already have superseded this ticket.
  void invoke("cancel_preview", { ticket }).catch(() => {});
}
