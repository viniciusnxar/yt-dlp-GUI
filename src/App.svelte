<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import * as api from "./lib/api";
  import { loadSettings, saveSettings } from "./lib/stores";
  import type { JobInfo, OptionDef, Selected } from "./lib/types";
  import OptionControl from "./components/OptionControl.svelte";
  import QueueItem from "./components/QueueItem.svelte";

  let options = $state<OptionDef[]>([]);
  let selected = $state<Selected>({});
  let folder = $state("");
  let url = $state("");
  let jobs = $state<JobInfo[]>([]);
  let preview = $state("");
  let error = $state("");
  let version = $state("");
  let updating = $state(false);
  let ready = $state(false);

  const groups = $derived([...new Set(options.map((o) => o.group))]);

  onMount(() => {
    let off: (() => void) | undefined;
    (async () => {
      options = await api.getOptions();
      const s = await loadSettings().catch(() => ({}) as Awaited<ReturnType<typeof loadSettings>>);
      const ids = new Set(options.map((o) => o.id));
      // Ignora IDs que não existem mais no registro (settings antigos).
      selected = s.selected
        ? Object.fromEntries(Object.entries(s.selected).filter(([k]) => ids.has(k)))
        : Object.fromEntries(options.filter((o) => o.default != null).map((o) => [o.id, o.default]));
      folder = s.folder || (await api.defaultFolder());
      jobs = await api.getQueue();
      off = await api.onDownloadEvent(onEvent);
      ready = true;
      version = await api.getYtdlpVersion().catch((e) => `erro: ${e}`);
    })();
    return () => off?.();
  });

  async function onEvent(kind: string, p: Partial<JobInfo> & { id: string; message?: string }) {
    if (kind === "queued") return void (jobs = await api.getQueue());
    const j = jobs.find((j) => j.id === p.id);
    if (!j) return;
    if (kind === "started") j.status = "running";
    else if (kind === "progress") Object.assign(j, { percent: p.percent, speed: p.speed, eta: p.eta });
    else if (kind === "title") j.title = p.title ?? null;
    else if (kind === "done") Object.assign(j, { status: "done", percent: 100, filepath: p.filepath });
    else if (kind === "error") Object.assign(j, { status: "error", error: p.message });
    else if (kind === "cancelled") j.status = "cancelled";
  }

  $effect(() => {
    if (!ready) return;
    const snap = $state.snapshot(selected) as Selected;
    api.previewCommand(snap).then((p) => ((preview = p), (error = ""))).catch((e) => (error = String(e)));
    saveSettings({ folder, selected: snap });
  });

  async function download() {
    error = "";
    const urls = url.split(/\s+/).filter(Boolean);
    try {
      for (const u of urls) await api.startDownload(u, $state.snapshot(selected) as Selected, folder);
      url = "";
    } catch (e) {
      error = String(e);
    }
  }

  async function pickFolder() {
    const p = await open({ directory: true, defaultPath: folder });
    if (typeof p === "string") folder = p;
  }

  async function update() {
    updating = true;
    try {
      const out = await api.updateYtdlp();
      version = await api.getYtdlpVersion();
      alert(out);
    } catch (e) {
      alert(String(e));
    } finally {
      updating = false;
    }
  }
</script>

<main>
  <form class="bar" onsubmit={(e) => (e.preventDefault(), download())}>
    <input bind:value={url} placeholder="Cole a URL aqui (várias separadas por espaço)" aria-label="URL" />
    <button type="submit" disabled={!url.trim()}>Baixar</button>
  </form>

  <div class="bar">
    <span class="folder" title={folder}>Pasta: {folder}</span>
    <button onclick={pickFolder}>Alterar</button>
  </div>

  {#if error}<p class="err">{error}</p>{/if}

  <div class="cols">
    <section class="opts">
      {#each groups as g}
        <details open>
          <summary>{g}</summary>
          {#each options.filter((o) => o.group === g) as def (def.id)}
            <OptionControl {def} bind:value={selected[def.id]} />
          {/each}
        </details>
      {/each}
    </section>

    <section class="queue">
      <h3>Fila</h3>
      {#if jobs.length === 0}<p class="muted">Nenhum download ainda.</p>{/if}
      <ul>
        {#each [...jobs].reverse() as job (job.id)}<QueueItem {job} />{/each}
      </ul>
    </section>
  </div>

  <footer>
    <code>$ {preview}</code>
    <button onclick={() => navigator.clipboard.writeText(preview)}>Copiar comando</button>
    <span class="muted">yt-dlp {version}</span>
    <button onclick={update} disabled={updating}>{updating ? "Atualizando…" : "Atualizar yt-dlp"}</button>
  </footer>
</main>

<style>
  :global(:root) {
    color-scheme: light dark;
    --bg: #fafafa; --fg: #1d1d1f; --border: #d0d0d5; --muted: #6e6e73; --danger: #c62828; --accent: #2962ff;
    font-family: system-ui, sans-serif; font-size: 14px;
  }
  @media (prefers-color-scheme: dark) {
    :global(:root) { --bg: #1c1c1e; --fg: #f2f2f7; --border: #3a3a3c; --muted: #98989d; --danger: #ef5350; --accent: #82b1ff; }
  }
  :global(body) { margin: 0; background: var(--bg); color: var(--fg); }
  :global(input, select, button) { font: inherit; color: inherit; background: transparent; border: 1px solid var(--border); border-radius: 4px; padding: .3rem .5rem; }
  :global(button:not(:disabled)) { cursor: pointer; }
  :global(button:hover:not(:disabled)) { border-color: var(--accent); }
  :global(select option) { background: var(--bg); }
  main { display: flex; flex-direction: column; height: 100vh; box-sizing: border-box; padding: .75rem; gap: .5rem; }
  .bar { display: flex; gap: .5rem; align-items: center; }
  .bar input { flex: 1; }
  .folder { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .cols { flex: 1; display: grid; grid-template-columns: minmax(300px, 1fr) 1fr; gap: .75rem; min-height: 0; }
  .opts, .queue { overflow: auto; border: 1px solid var(--border); border-radius: 6px; padding: .5rem; }
  summary { font-weight: 600; cursor: pointer; margin: .3rem 0; }
  details > :global(.row) { margin-left: 1rem; }
  h3 { margin: 0 0 .5rem; }
  ul { padding: 0; margin: 0; }
  footer { display: flex; gap: .5rem; align-items: center; }
  footer code { flex: 1; overflow-x: auto; white-space: nowrap; padding: .3rem; border: 1px solid var(--border); border-radius: 4px; }
  .muted { color: var(--muted); }
  .err { color: var(--danger); margin: 0; }
</style>
