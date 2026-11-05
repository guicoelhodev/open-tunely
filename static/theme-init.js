// Apply saved colors before the page is painted, including before Svelte loads.
(() => {
  let palette = "indigo";
  let mode = "light";
  try {
    const saved = JSON.parse(localStorage.getItem("tunely.appearance") ?? "null");
    if (["pink", "indigo", "cyan", "neutral"].includes(saved?.palette)) palette = saved.palette;
    if (["light", "dark"].includes(saved?.mode)) mode = saved.mode;
  } catch {
    // Invalid or unavailable storage must not prevent the app from starting.
  }
  const root = document.documentElement;
  root.dataset.theme = palette;
  root.classList.toggle("dark", mode === "dark");
  root.style.colorScheme = mode;
})();
