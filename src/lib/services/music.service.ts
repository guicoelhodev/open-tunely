import { m } from "$lib/paraglide/messages.js";
import { invoke, isTauri } from "@tauri-apps/api/core";

export type Song = { id: string; title: string; channel?: string | null };

export function requireDesktop() {
  if (!isTauri()) throw new Error(m.text_open_the_desktop_app_to_use_this_feature());
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
