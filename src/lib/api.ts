// Wrappers tipados de invoke() e listen().
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { JobInfo, OptionDef, Selected } from "./types";

export const getOptions = () => invoke<OptionDef[]>("get_options");
export const previewCommand = (selected: Selected) => invoke<string>("preview_command", { selected });
export const startDownload = (url: string, selected: Selected, folder: string) =>
  invoke<string>("start_download", { url, selected, folder });
export const cancelDownload = (jobId: string) => invoke<void>("cancel_download", { jobId });
export const getQueue = () => invoke<JobInfo[]>("get_queue");
export const defaultFolder = () => invoke<string>("default_folder");
export const updateYtdlp = () => invoke<string>("update_ytdlp");
export const getYtdlpVersion = () => invoke<string>("get_ytdlp_version");

/** Escuta todos os eventos download://* e entrega (evento, payload). */
export async function onDownloadEvent(cb: (kind: string, p: Partial<JobInfo> & { id: string; message?: string }) => void) {
  const kinds = ["queued", "started", "progress", "title", "done", "error", "cancelled"];
  const offs = await Promise.all(kinds.map((k) => listen<any>(`download://${k}`, (e) => cb(k, e.payload))));
  return () => offs.forEach((off) => off());
}
