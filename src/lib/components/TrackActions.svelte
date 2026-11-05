<script lang="ts">
  import { errorMessage } from "$lib/services/errors.service";
  import { locale } from "$lib/services/locale.service";
  import { m } from "$lib/paraglide/messages.js";
  import Button from "./ui/Button.svelte";
  import Icon from "@iconify/svelte";
  import Popover from "./ui/Popover.svelte";
  import Modal from "./ui/Modal.svelte";
  import trashIcon from "@iconify-icons/andrsrxn/trash";
  import editIcon from "@iconify-icons/andrsrxn/pencil";
  import { library, renameMusicTrack, deleteMusicTrack, refreshMusicLibrary, type Track } from "$lib/services/library.service";
  import { tick } from "svelte";

  let { onbeforechange, onnotice }: {
    onbeforechange: (track: Track) => void;
    onnotice: (message: string) => void;
  } = $props();
  let menu = $state<HTMLDivElement>(undefined!);
  let dialog = $state<HTMLDialogElement>(undefined!);
  let input = $state<HTMLInputElement>();
  let selected = $state<Track | null>(null);
  let action = $state<"rename" | "delete">("rename");
  let name = $state("");
  let error = $state("");
  let busy = $state(false);
  let keyboardFocus = $state(false);
  let origin: HTMLElement | null = null;
  let folder = "";

  export async function open(event: MouseEvent | KeyboardEvent, track: Track) {
    if (busy || dialog.open) return;
    event.preventDefault();
    const target = event.currentTarget as HTMLElement;
    origin = target.matches("button") ? target : target.querySelector("button");
    selected = track;
    keyboardFocus = event instanceof KeyboardEvent;
    folder = $library?.folder ?? "";
    menu.showPopover();
    const bounds = target.getBoundingClientRect();
    const x = event instanceof MouseEvent ? event.clientX : bounds.left;
    const y = event instanceof MouseEvent ? event.clientY : bounds.bottom;
    menu.style.left = `${Math.max(8, Math.min(x, window.innerWidth - menu.offsetWidth - 8))}px`;
    menu.style.top = `${Math.max(8, Math.min(y, window.innerHeight - menu.offsetHeight - 8))}px`;
    menu.querySelector<HTMLButtonElement>("button")?.focus();
  }

  function closeMenu() { menu?.hidePopover(); }
  function outsidePointer(event: PointerEvent) {
    if (menu?.matches(":popover-open") && !event.composedPath().includes(menu)) closeMenu();
  }
  function escapeMenu(event: KeyboardEvent) {
    if (event.key === "Escape" && menu?.matches(":popover-open")) {
      event.preventDefault();
      closeMenu();
      origin?.focus();
    }
  }
  export function dismiss() { closeMenu(); }
  function menuKeys(event: KeyboardEvent) {
    keyboardFocus = true;
    const buttons = [...menu.querySelectorAll<HTMLButtonElement>("button")];
    if (["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) {
      event.preventDefault();
      const index = buttons.indexOf(document.activeElement as HTMLButtonElement);
      const next = event.key === "Home" ? 0 : event.key === "End" ? buttons.length - 1
        : (index + (event.key === "ArrowDown" ? 1 : -1) + buttons.length) % buttons.length;
      buttons[next]?.focus();
    } else if (event.key === "Escape") {
      closeMenu(); origin?.focus();
    }
  }

  async function edit(next: "rename" | "delete") {
    closeMenu();
    action = next;
    name = selected?.title ?? "";
    error = "";
    dialog.showModal();
    await tick();
    if (next === "rename") { input?.focus(); input?.select(); }
  }

  function cancel() { if (!busy) dialog.close(); }
  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!selected || busy) return;
    if ($library?.folder !== folder) {
      error = m.text_the_selected_folder_changed_close_this_dialog_and_select_the_song_again({}, { locale: $locale });
      return;
    }
    busy = true;
    error = "";
    try {
      onbeforechange(selected);
      if (action === "rename") await renameMusicTrack(selected, name);
      else await deleteMusicTrack(selected);
    } catch (cause) {
      error = String(cause);
      busy = false;
      return;
    }
    dialog.close();
    try {
      await refreshMusicLibrary();
      onnotice("");
    } catch {
      onnotice(m.text_file_changed_select_the_folder_again_to_refresh_the_library({}, { locale: $locale }));
    } finally { busy = false; }
  }
</script>

<svelte:window onresize={closeMenu} onblur={closeMenu} onpointerdown={outsidePointer} onkeydown={escapeMenu} />

<Popover class="inset-auto w-[min(144px,calc(100vw-32px))] rounded-md p-1 shadow-[0_8px_28px_#17202a26] {keyboardFocus ? '' : 'outline-none [&_button]:outline-none'}" bind:element={menu} popover="manual" role="menu" tabindex={-1} aria-label={m.text_song_actions({}, { locale: $locale })} onkeydown={menuKeys} onpointerdown={() => keyboardFocus = false}>
  <Button variant="option" size="custom" layout="menu" class="w-full px-[9px] py-[7px] text-[0.8rem]" type="button" role="menuitem" onclick={() => edit("rename")}><Icon icon={editIcon} width="16" />{m.text_rename({}, { locale: $locale })}</Button>
  <Button variant="danger" size="custom" layout="menu" class="mt-0.5 w-full px-[9px] py-[7px] text-[0.8rem]" type="button" role="menuitem" onclick={() => edit("delete")}><Icon icon={trashIcon} width="16" />{m.text_delete({}, { locale: $locale })}</Button>
</Popover>

<Modal bind:element={dialog} oncancel={(event) => { if (busy) event.preventDefault(); }} onclose={() => origin?.focus()} aria-labelledby="track-action-title">
  <form onsubmit={submit}>
    <h3 class="mb-5 text-base font-bold" id="track-action-title">{action === "rename" ? m.text_rename_song({}, { locale: $locale }) : m.text_delete_song({}, { locale: $locale })}</h3>
    {#if action === "rename"}
      <label class="mb-2 block text-[0.85rem]" for="track-name">{m.text_song_name({}, { locale: $locale })}</label>
      <input class="w-full rounded-md border border-input p-2.5 focus-visible:outline-3 focus-visible:outline-solid focus-visible:outline-ring focus-visible:outline-offset-2" bind:this={input} id="track-name" bind:value={name} required maxlength="240" disabled={busy} />
      <p class="my-4 text-[0.85rem] leading-[1.6] [overflow-wrap:anywhere]">{m.text_the_mp3_extension_will_be_kept({}, { locale: $locale })}</p>
    {:else}
      <p class="my-4 text-[0.85rem] leading-[1.6] [overflow-wrap:anywhere]">{m.delete_confirm({ title: selected?.title ?? "" }, { locale: $locale })}</p>
    {/if}
    {#if error}<p class="my-4 text-[0.85rem] leading-[1.6] text-red-700 dark:text-red-300 [overflow-wrap:anywhere]" role="alert">{errorMessage(error, $locale)}</p>{/if}
    <div class="mt-5 flex justify-end gap-2">
      <Button variant="muted" type="button" onclick={cancel} disabled={busy}>{m.text_cancel({}, { locale: $locale })}</Button>
      <Button variant={action === "delete" ? "danger" : "primary"} type="submit" disabled={busy || (action === "rename" && !name.trim())}>
        {busy ? m.text_please_wait({}, { locale: $locale }) : action === "rename" ? m.text_save({}, { locale: $locale }) : m.text_delete({}, { locale: $locale })}
      </Button>
    </div>
  </form>
</Modal>
