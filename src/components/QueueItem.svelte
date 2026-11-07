<script lang="ts">
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { cancelDownload } from "../lib/api";
  import type { JobInfo } from "../lib/types";

  let { job }: { job: JobInfo } = $props();
  const label: Record<string, string> = { queued: "Na fila", done: "Concluído", cancelled: "Cancelado" };
</script>

<li class={job.status}>
  <div class="title" title={job.url}>{job.title ?? job.url}</div>
  <progress max="100" value={job.percent}></progress>
  <div class="meta">
    <span>
      {#if job.status === "running"}{job.percent.toFixed(1)}% · {job.speed || "…"} · {job.eta || "…"}
      {:else if job.status === "error"}<span class="err">{job.error}</span>
      {:else}{label[job.status]}{/if}
    </span>
    {#if job.status === "running" || job.status === "queued"}
      <button onclick={() => cancelDownload(job.id)} aria-label="Cancelar">✕</button>
    {:else if job.status === "done" && job.filepath}
      <button onclick={() => revealItemInDir(job.filepath!)}>Abrir pasta</button>
    {/if}
  </div>
</li>

<style>
  li { list-style: none; padding: .5rem; border: 1px solid var(--border); border-radius: 6px; margin-bottom: .5rem; }
  .title { font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  progress { width: 100%; }
  .meta { display: flex; justify-content: space-between; align-items: center; gap: .5rem; font-size: .85em; }
  .err { color: var(--danger); }
  .cancelled, .error { opacity: .75; }
</style>
