<script lang="ts">
  import { invoke, isTauri } from "@tauri-apps/api/core";
  type Song = { id: string; title: string };
  let query = $state("");
  let song = $state<Song | null>(null);
  let loading = $state(false);
  let error = $state("");

  async function search(event: SubmitEvent) {
    event.preventDefault();
    if (loading || !query.trim()) return;
    if (!isTauri()) {
      error = "Abra o aplicativo desktop para pesquisar. Inicie com npm run dev e use a janela do aplicativo.";
      return;
    }
    loading = true;
    song = null;
    error = "";
    try {
      song = await invoke<Song>("search_song", { query });
    } catch (cause) {
      error = String(cause);
    } finally {
      loading = false;
    }
  }
</script>

<main>
  <h1>Pesquisar música</h1>
  <p>Encontre uma música no YouTube pelo nome ou artista.</p>
  <form onsubmit={search}>
    <label for="song-query">Música ou artista</label>
    <div class="search-row">
      <input id="song-query" type="search" bind:value={query}
        placeholder="Ex.: Djavan — Oceano" required maxlength="300" />
      <button type="submit" disabled={loading || !query.trim()}>
        {loading ? "Pesquisando…" : "Pesquisar"}
      </button>
    </div>
  </form>
  <div aria-live="polite" aria-busy={loading}>
    {#if loading}
      <p>Buscando no YouTube…</p>
    {:else if error}
      <p class="error" role="alert">{error}</p>
    {:else if song}
      <article>
        <p class="caption">Primeiro resultado do YouTube</p>
        <h2>{song.title}</h2>
      </article>
    {/if}
  </div>
</main>

<style>
  :global(body) { margin: 0; background: #f6f6f6; color: #17202a; font-family: Inter, system-ui, sans-serif; }
  main { max-width: 640px; margin: 10vh auto; padding: 24px; }
  h1 { margin-bottom: 8px; }
  p { line-height: 1.6; }
  form { margin-top: 32px; }
  label { display: block; margin-bottom: 8px; font-weight: 600; }
  .search-row { display: flex; gap: 12px; flex-wrap: wrap; }
  input, button { font: inherit; padding: 12px 16px; border-radius: 8px; }
  input { flex: 1; min-width: 180px; border: 1px solid #8b949e; background: white; }
  button { border: 0; background: #2458d3; color: white; cursor: pointer; }
  button:disabled { opacity: .6; cursor: default; }
  input:focus-visible, button:focus-visible { outline: 3px solid #7aa7ff; outline-offset: 3px; }
  article { margin-top: 24px; padding: 20px; border: 1px solid #d8dee7; border-radius: 12px; background: white; }
  .caption { margin: 0 0 8px; color: #536170; font-size: .875rem; }
  h2 { margin: 0; font-size: 1.2rem; overflow-wrap: anywhere; }
  .error { color: #a51e25; }
</style>
