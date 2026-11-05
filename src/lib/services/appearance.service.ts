import { get, readonly, writable } from "svelte/store";

export const palettes = [
  { id: "neutral", label: "Neutro", color: "#525252", darkColor: "#d4d4d4" },
  { id: "cyan", label: "Ciano", color: "#0891b2", darkColor: "#22d3ee" },
  { id: "indigo", label: "Índigo", color: "#4f46e5", darkColor: "#a5b4fc" },
  { id: "pink", label: "Rosa", color: "#e63e93", darkColor: "#ff69b4" },
] as const;

export type Palette = typeof palettes[number]["id"];
type Appearance = { palette: Palette; mode: "light" | "dark" };

function initialAppearance(): Appearance {
  const root = typeof document !== "undefined" ? document.documentElement : undefined;
  const palette = palettes.find(item => item.id === root?.dataset.theme)?.id ?? "indigo";
  return { palette, mode: root?.classList.contains("dark") ? "dark" : "light" };
}

const current = writable<Appearance>(initialAppearance());
export const appearance = readonly(current);

function apply(value: Appearance) {
  const root = document.documentElement;
  root.dataset.theme = value.palette;
  root.classList.toggle("dark", value.mode === "dark");
  root.style.colorScheme = value.mode;
  current.set(value);
  try {
    localStorage.setItem("tunely.appearance", JSON.stringify(value));
  } catch {
    // Keep the selected appearance usable when storage is unavailable.
  }
}

export function selectPalette(palette: Palette) {
  apply({ ...get(current), palette });
}

export function toggleColorMode() {
  const value = get(current);
  apply({ ...value, mode: value.mode === "dark" ? "light" : "dark" });
}
