// Settings persistidos (pasta + opções marcadas) via tauri-plugin-store.
import { load } from "@tauri-apps/plugin-store";
import type { Selected } from "./types";

export type Settings = { folder: string; selected: Selected };

const store = load("settings.json", { autoSave: 300, defaults: {} });

export async function loadSettings(): Promise<Partial<Settings>> {
  const s = await store;
  return { folder: await s.get<string>("folder"), selected: await s.get<Selected>("selected") };
}

export async function saveSettings(v: Settings) {
  const s = await store;
  await s.set("folder", v.folder);
  await s.set("selected", v.selected);
}
