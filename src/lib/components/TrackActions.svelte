<script lang="ts">
  import Icon from "@iconify/svelte";
  import trashIcon from "@iconify-icons/andrsrxn/trash";
  import editIcon from "@iconify-icons/andrsrxn/pencil";
  import { library, renameMusicTrack, deleteMusicTrack, refreshMusicLibrary, type Track } from "$lib/services/library.service";
  import { tick } from "svelte";

  let { onbeforechange, onnotice }: {
    onbeforechange: (track: Track) => void;
    onnotice: (message: string) => void;
  } = $props();
  let menu: HTMLDivElement;
  let dialog: HTMLDialogElement;
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
      error = "A pasta selecionada mudou. Feche este diálogo e escolha a música novamente.";
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
      onnotice("Arquivo alterado. Selecione a pasta novamente para atualizar a biblioteca.");
    } finally { busy = false; }
  }
</script>

<svelte:window onresize={closeMenu} onblur={closeMenu} onpointerdown={outsidePointer} onkeydown={escapeMenu} />

<div class="track-menu" class:keyboard-focus={keyboardFocus} bind:this={menu} popover="manual" role="menu" tabindex="-1" aria-label="Ações da música" onkeydown={menuKeys} onpointerdown={() => keyboardFocus = false}>
  <button type="button" role="menuitem" onclick={() => edit("rename")}><Icon icon={editIcon} width="16" />Renomear</button>
  <button class="delete" type="button" role="menuitem" onclick={() => edit("delete")}><Icon icon={trashIcon} width="16" />Excluir</button>
</div>

<dialog bind:this={dialog} oncancel={(event) => { if (busy) event.preventDefault(); }} onclose={() => origin?.focus()} aria-labelledby="track-action-title">
  <form onsubmit={submit}>
    <h3 id="track-action-title">{action === "rename" ? "Renomear música" : "Excluir música"}</h3>
    {#if action === "rename"}
      <label for="track-name">Nome da música</label>
      <input bind:this={input} id="track-name" bind:value={name} required maxlength="240" disabled={busy} />
      <p>A extensão .mp3 será mantida.</p>
    {:else}
      <p>Excluir “{selected?.title}” da pasta? O arquivo será apagado permanentemente.</p>
    {/if}
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    <div class="dialog-actions">
      <button type="button" onclick={cancel} disabled={busy}>Cancelar</button>
      <button class:delete={action === "delete"} class:save={action === "rename"} type="submit" disabled={busy || (action === "rename" && !name.trim())}>
        {busy ? "Aguarde…" : action === "rename" ? "Salvar" : "Excluir"}
      </button>
    </div>
  </form>
</dialog>

<style>
  button, input { font: inherit; }
  button { display: flex; align-items: center; gap: 10px; border: 0; border-radius: 8px; padding: 10px 12px; background: transparent; color: #34445a; cursor: pointer; }
  button:hover { background: #edf3ff; }
  button:focus-visible, input:focus-visible { outline: 3px solid #7aa7ff; outline-offset: 2px; }
  button:disabled { opacity: .5; cursor: wait; }
  .track-menu { position: fixed; inset: auto; margin: 0; width: min(144px, calc(100vw - 32px)); padding: 4px; box-sizing: border-box; border: 1px solid #d8dee7; border-radius: 9px; background: white; box-shadow: 0 8px 28px #17202a26; }
  .track-menu button { width: 100%; text-align: left; gap: 8px; padding: 7px 9px; border-radius: 6px; font-size: .8rem; }
  .track-menu:not(.keyboard-focus), .track-menu:not(.keyboard-focus) button { outline: none; }
  .track-menu button + button { margin-top: 2px; }
  .delete { color: #a51e25; background: #fcebed; }
  .delete:hover { background: #f8dce0; }
  dialog { width: min(360px, calc(100vw - 64px)); border: 1px solid #d8dee7; border-radius: 16px; padding: 24px; color: #17202a; }
  dialog::backdrop { background: #17202a55; }
  h3 { margin: 0 0 20px; font-size: 1rem; }
  label { display: block; margin-bottom: 8px; font-size: .85rem; }
  input { box-sizing: border-box; width: 100%; border: 1px solid #8b949e; border-radius: 8px; padding: 10px; }
  p { font-size: .85rem; line-height: 1.6; overflow-wrap: anywhere; }
  .error { color: #a51e25; }
  .dialog-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 20px; }
  .save { background: #2458d3; color: white; }
  .save:hover { background: #1946b1; }
</style>
