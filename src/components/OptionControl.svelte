<script lang="ts">
  import { save } from "@tauri-apps/plugin-dialog";
  import type { OptionDef } from "../lib/types";

  let { def, value = $bindable() }: { def: OptionDef; value: unknown } = $props();
  const k = $derived(def.kind);
  const id = $derived(`opt-${def.id}`);

  async function pickFile() {
    const p = await save({ title: def.label });
    if (p) value = p;
  }
</script>

<div class="row" title={def.help}>
  {#if k.type === "toggle"}
    <label><input type="checkbox" checked={value === true} onchange={(e) => (value = e.currentTarget.checked)} /> {def.label}</label>
  {:else}
    <label for={id}>{def.label}</label>
    {#if k.type === "select" || k.type === "preset"}
      <select {id} value={value ?? ""} onchange={(e) => (value = e.currentTarget.value || null)}>
        <option value="">—</option>
        {#each k.type === "select" ? k.choices.map((c) => [c.value, c.label]) : k.choices.map((c) => [c.id, c.label]) as [v, l]}
          <option value={v}>{l}</option>
        {/each}
      </select>
    {:else if k.type === "number"}
      <input {id} type="number" min={k.min} max={k.max} value={value ?? ""}
        oninput={(e) => (value = e.currentTarget.value === "" ? null : Number(e.currentTarget.value))} />
    {:else}
      <span class="field">
        <input {id} type="text" placeholder={k.type === "text" ? k.placeholder : ""} value={value ?? ""}
          oninput={(e) => (value = e.currentTarget.value || null)} />
        {#if k.type === "path"}<button type="button" onclick={pickFile}>…</button>{/if}
      </span>
    {/if}
  {/if}
</div>

<style>
  .row { display: grid; grid-template-columns: 9rem 1fr; align-items: center; gap: .5rem; margin: .3rem 0; }
  .row:has(> label:only-child) { grid-template-columns: 1fr; }
  .field { display: flex; gap: .3rem; }
  .field input { flex: 1; min-width: 0; }
</style>
