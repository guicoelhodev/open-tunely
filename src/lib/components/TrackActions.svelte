<script lang="ts">
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

<Popover class="inset-auto w-[min(144px,calc(100vw-32px))] rounded-[9px] p-1 shadow-[0_8px_28px_#17202a26] {keyboardFocus ? '' : 'outline-none [&_button]:outline-none'}" bind:element={menu} popover="manual" role="menu" tabindex={-1} aria-label="Ações da música" onkeydown={menuKeys} onpointerdown={() => keyboardFocus = false}>
  <button class="flex cursor-pointer items-center bg-transparent text-[#34445a] hover:bg-[#edf3ff] focus-visible:outline-3 focus-visible:outline-solid focus-visible:outline-[#7aa7ff] focus-visible:outline-offset-2 disabled:cursor-wait disabled:opacity-50 w-full gap-2 rounded-md px-[9px] py-[7px] text-left text-[0.8rem]" type="button" role="menuitem" onclick={() => edit("rename")}><Icon icon={editIcon} width="16" />Renomear</button>
  <button class="flex cursor-pointer items-center focus-visible:outline-3 focus-visible:outline-solid focus-visible:outline-[#7aa7ff] focus-visible:outline-offset-2 disabled:cursor-wait disabled:opacity-50 w-full gap-2 rounded-md px-[9px] py-[7px] text-left text-[0.8rem] mt-0.5 bg-[#fcebed] text-[#a51e25] hover:bg-[#f8dce0]" type="button" role="menuitem" onclick={() => edit("delete")}><Icon icon={trashIcon} width="16" />Excluir</button>
</Popover>

<Modal bind:element={dialog} oncancel={(event) => { if (busy) event.preventDefault(); }} onclose={() => origin?.focus()} aria-labelledby="track-action-title">
  <form onsubmit={submit}>
    <h3 class="mb-5 text-base font-bold" id="track-action-title">{action === "rename" ? "Renomear música" : "Excluir música"}</h3>
    {#if action === "rename"}
      <label class="mb-2 block text-[0.85rem]" for="track-name">Nome da música</label>
      <input class="w-full rounded-lg border border-[#8b949e] p-2.5 focus-visible:outline-3 focus-visible:outline-solid focus-visible:outline-[#7aa7ff] focus-visible:outline-offset-2" bind:this={input} id="track-name" bind:value={name} required maxlength="240" disabled={busy} />
      <p class="my-4 text-[0.85rem] leading-[1.6] [overflow-wrap:anywhere]">A extensão .mp3 será mantida.</p>
    {:else}
      <p class="my-4 text-[0.85rem] leading-[1.6] [overflow-wrap:anywhere]">Excluir “{selected?.title}” da pasta? O arquivo será apagado permanentemente.</p>
    {/if}
    {#if error}<p class="my-4 text-[0.85rem] leading-[1.6] text-[#a51e25] [overflow-wrap:anywhere]" role="alert">{error}</p>{/if}
    <div class="mt-5 flex justify-end gap-2">
      <button class="flex cursor-pointer items-center gap-2.5 rounded-lg bg-transparent px-3 py-2.5 text-[#34445a] hover:bg-[#edf3ff] focus-visible:outline-3 focus-visible:outline-solid focus-visible:outline-[#7aa7ff] focus-visible:outline-offset-2 disabled:cursor-wait disabled:opacity-50" type="button" onclick={cancel} disabled={busy}>Cancelar</button>
      <button class="flex cursor-pointer items-center gap-2.5 rounded-lg px-3 py-2.5 focus-visible:outline-3 focus-visible:outline-solid focus-visible:outline-[#7aa7ff] focus-visible:outline-offset-2 disabled:cursor-wait disabled:opacity-50 {action === 'delete' ? 'bg-[#fcebed] text-[#a51e25] hover:bg-[#f8dce0]' : 'bg-[#2458d3] text-white hover:bg-[#1946b1]'}" type="submit" disabled={busy || (action === "rename" && !name.trim())}>
        {busy ? "Aguarde…" : action === "rename" ? "Salvar" : "Excluir"}
      </button>
    </div>
  </form>
</Modal>
