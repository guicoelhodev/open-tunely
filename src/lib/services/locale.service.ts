import { writable } from "svelte/store";
import { getLocale, setLocale, type Locale } from "$lib/paraglide/runtime.js";

export const locale = writable<Locale>(getLocale());

export function selectLocale(value: Locale) {
  setLocale(value, { reload: false });
  locale.set(value);
  document.documentElement.lang = value;
}
