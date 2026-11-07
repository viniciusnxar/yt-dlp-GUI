# YT-DLP GUI — Especificação do Projeto

> Documento de referência para implementação com Claude Code.
> Objetivo: um app desktop **leve** que é apenas uma interface gráfica para o `yt-dlp`, com checkboxes/selects para as opções mais usadas (ex.: `--cookies-from-browser`).

---

## 1. Decisões já tomadas

| Tema | Decisão | Motivo |
|---|---|---|
| Framework | **Tauri v2** | Binário pequeno (~3–10 MB), baixo uso de RAM, UI em web, ecossistema maduro (sidecar, plugins, instaladores) |
| Alternativas descartadas | Electron (pesado), GPUI (pouca doc, overkill), iced/Dioxus (válidos, mas Tauri tem menos fricção) | — |
| Front-end | **Svelte + TypeScript + Vite** (sem React) | Leve e suficiente para uma tela de formulário |
| Lógica de download | **Não reimplementar nada** — apenas executar o binário `yt-dlp` e ler o stdout | yt-dlp muda com frequência; o app é só uma casca |
| Binários | `yt-dlp` e `ffmpeg` empacotados como **sidecar** | Usuário não precisa instalar nada |

---

## 2. Stack

- **Tauri v2** (Rust)
- **Svelte + TypeScript + Vite**
- Plugins Tauri:
  - `tauri-plugin-shell` — executar sidecars
  - `tauri-plugin-store` — persistir configurações
  - `tauri-plugin-dialog` — escolher pasta de destino
  - `tauri-plugin-opener` — abrir arquivo/pasta ao terminar (opcional)
- Crates Rust: `serde`, `serde_json`, `tokio` (já vem com Tauri), `uuid` (IDs dos jobs), `thiserror` (erros)

---

## 3. Estrutura de pastas

```
yt-gui/
├── src/                          # Front-end (UI)
│   ├── App.svelte                # Layout: URL, painel de opções, fila
│   ├── components/
│   │   ├── UrlInput.svelte       # Campo de URL + botão "Baixar"
│   │   ├── OptionsPanel.svelte   # Gera controles a partir do registro de opções
│   │   ├── OptionControl.svelte  # Renderiza 1 opção (toggle/select/text/path/number)
│   │   ├── CommandPreview.svelte # Mostra o comando final (somente leitura, copiável)
│   │   ├── QueueList.svelte      # Lista de downloads
│   │   └── QueueItem.svelte      # Barra de progresso, velocidade, ETA, cancelar
│   └── lib/
│       ├── api.ts                # Wrappers tipados de invoke() e listen()
│       ├── types.ts              # Tipos espelhando os structs do Rust
│       └── stores.ts             # Estado: opções selecionadas, fila, settings
│
└── src-tauri/
    ├── binaries/                 # Sidecars (nome com target triple, ver seção 9)
    ├── capabilities/default.json
    ├── tauri.conf.json
    ├── Cargo.toml
    └── src/
        ├── main.rs               # Bootstrap, registra plugins, state e commands
        ├── commands.rs           # Funções #[tauri::command] expostas ao front
        ├── options.rs            # Registro de flags (fonte única da verdade)
        ├── args.rs               # Opções selecionadas -> Vec<String>
        ├── runner.rs             # Spawn/kill do sidecar, leitura do stdout/stderr
        ├── progress.rs           # Parser das linhas de progresso
        ├── queue.rs              # Fila de jobs + limite de concorrência
        ├── settings.rs           # Pasta padrão, navegador, presets
        └── error.rs              # Tipo de erro serializável para o front
```

---

## 4. Fluxo de um download

```
[UI] usuário cola URL e marca opções
   │ invoke("start_download", { url, selected })
   ▼
[commands.rs] valida: só aceita IDs que existem em options.rs
   ▼
[args.rs] gera Vec<String>, ex.:
   ["-f","bv*+ba/b","--cookies-from-browser","firefox",
    "--newline","--progress","--progress-template", TEMPLATE,
    "--print","after_move:filepath",
    "--ffmpeg-location", <caminho do sidecar ffmpeg>,
    "-P", <pasta>, <url>]
   ▼
[queue.rs] cria Job { id, url, status: Queued } e dispara quando houver vaga
   ▼
[runner.rs] spawn do sidecar yt-dlp; lê stdout/stderr linha a linha
   ▼
[progress.rs] linha "dl|42.1%|3.20MiB/s|00:15" -> Progress { percent, speed, eta }
   │ emit("download://progress", { id, percent, speed, eta })
   ▼
[UI] atualiza a barra
   │
   └─ fim: emit("download://done", { id, filepath })
          ou emit("download://error", { id, message, stderr_tail })
```

---

## 5. Registro de opções (`options.rs`)

Fonte única da verdade. O front busca via `get_options()` e **gera a UI automaticamente**. Adicionar uma flag = adicionar uma entrada aqui.

```rust
#[derive(Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OptKind {
    Toggle,                                   // flag sem valor
    Select { choices: Vec<Choice> },          // flag + valor de uma lista
    Text { placeholder: String },             // flag + valor livre
    Number { min: i64, max: i64 },
    Path,                                     // flag + caminho (dialog)
    Preset { choices: Vec<PresetChoice> },    // escolha que gera VÁRIOS args
}

#[derive(Serialize, Clone)]
pub struct Choice { pub value: String, pub label: String }

#[derive(Serialize, Clone)]
pub struct PresetChoice { pub id: String, pub label: String, pub args: Vec<String> }

#[derive(Serialize, Clone)]
pub struct OptionDef {
    pub id: String,          // "cookies_browser"
    pub flag: String,        // "--cookies-from-browser" (vazio para Preset)
    pub label: String,       // "Usar cookies do navegador"
    pub help: String,        // tooltip
    pub group: String,       // "Formato", "Autenticação", "Legendas"...
    pub kind: OptKind,
    pub default: Option<serde_json::Value>,
}
```

### Opções iniciais sugeridas

| Grupo | id | Flag(s) | Tipo | Valores / observação |
|---|---|---|---|---|
| Formato | `quality` | — | Preset | Melhor: `-f bv*+ba/b` · Até 1080p: `-f bv*[height<=1080]+ba/b` · Até 720p: `-f bv*[height<=720]+ba/b` · Só áudio MP3: `-x --audio-format mp3` · Só áudio M4A: `-x --audio-format m4a` |
| Formato | `merge_format` | `--merge-output-format` | Select | `mp4`, `mkv`, `webm` |
| Autenticação | `cookies_browser` | `--cookies-from-browser` | Select | `firefox`, `chrome`, `edge`, `brave`, `opera`, `vivaldi`, `chromium` |
| Legendas | `write_subs` | `--write-subs` | Toggle | — |
| Legendas | `write_auto_subs` | `--write-auto-subs` | Toggle | Legendas automáticas |
| Legendas | `sub_langs` | `--sub-langs` | Text | ex.: `pt.*,en` |
| Legendas | `embed_subs` | `--embed-subs` | Toggle | — |
| Metadados | `embed_thumbnail` | `--embed-thumbnail` | Toggle | Requer ffmpeg |
| Metadados | `embed_metadata` | `--embed-metadata` | Toggle | — |
| Metadados | `embed_chapters` | `--embed-chapters` | Toggle | — |
| SponsorBlock | `sponsorblock_remove` | `--sponsorblock-remove` | Select | `sponsor`, `sponsor,selfpromo,intro,outro`, `all` |
| Playlist | `no_playlist` | `--no-playlist` | Toggle | Baixa só o vídeo mesmo se URL tiver `list=` |
| Playlist | `playlist_items` | `--playlist-items` | Text | ex.: `1-5,8` |
| Arquivo | `output_template` | `-o` | Text | padrão: `%(title)s [%(id)s].%(ext)s` |
| Arquivo | `restrict_filenames` | `--restrict-filenames` | Toggle | Só ASCII, sem espaços |
| Arquivo | `download_archive` | `--download-archive` | Path | Pula vídeos já baixados |
| Rede | `limit_rate` | `-r` | Text | ex.: `5M` |
| Rede | `concurrent_fragments` | `-N` | Number | 1–16 |

### Flags fixas (sempre adicionadas pelo backend, nunca expostas)

```
--newline
--progress
--progress-template "download:dl|%(progress._percent_str)s|%(progress._speed_str)s|%(progress._eta_str)s"
--print after_move:filepath
--ffmpeg-location <caminho do sidecar ffmpeg>
-P <pasta de destino>
```

> Observação: `--print` implica `--quiet`, por isso o `--progress` é obrigatório para continuar recebendo progresso.

---

## 6. Comandos Tauri (`commands.rs`)

| Comando | Entrada | Saída | Descrição |
|---|---|---|---|
| `get_options` | — | `Vec<OptionDef>` | Registro de flags para montar a UI |
| `preview_command` | `selected: HashMap<String, Value>` | `String` | Comando final legível, para exibir na UI |
| `start_download` | `url: String, selected: HashMap<String, Value>` | `JobId` | Valida, enfileira e retorna o ID |
| `cancel_download` | `job_id: String` | `()` | Mata o processo filho |
| `get_queue` | — | `Vec<JobInfo>` | Estado atual da fila (útil ao recarregar a UI) |
| `update_ytdlp` | — | `String` | Roda `yt-dlp -U` e retorna a saída |
| `get_ytdlp_version` | — | `String` | `yt-dlp --version` |
| `get_settings` / `save_settings` | `Settings` | `Settings` | Persistência via `tauri-plugin-store` |

## 7. Eventos (Rust → front)

| Evento | Payload |
|---|---|
| `download://queued` | `{ id, url }` |
| `download://started` | `{ id }` |
| `download://progress` | `{ id, percent: f32, speed: String, eta: String }` |
| `download://done` | `{ id, filepath: String }` |
| `download://error` | `{ id, message: String, stderr_tail: Vec<String> }` |
| `download://cancelled` | `{ id }` |

## 8. Estado global

```rust
pub struct AppState {
    pub jobs: Mutex<HashMap<String, JobInfo>>,
    pub children: Mutex<HashMap<String, CommandChild>>, // para cancelar
    pub max_concurrent: AtomicUsize,                     // ex.: 2
}
// registrado em main.rs com app.manage(AppState::default())
```

---

## 9. Sidecars (yt-dlp e ffmpeg)

- Colocar em `src-tauri/binaries/` com o **target triple** no nome:
  - `yt-dlp-x86_64-pc-windows-msvc.exe`
  - `ffmpeg-x86_64-pc-windows-msvc.exe`
  - (Linux: `x86_64-unknown-linux-gnu`; macOS: `aarch64-apple-darwin` / `x86_64-apple-darwin`)
- Descobrir o triple com: `rustc --print host-tuple` (ou `rustc -Vv` → linha `host`).
- `tauri.conf.json`:
  ```json
  { "bundle": { "externalBin": ["binaries/yt-dlp", "binaries/ffmpeg"] } }
  ```
- Executar pelo Rust:
  ```rust
  use tauri_plugin_shell::{ShellExt, process::CommandEvent};

  let (mut rx, child) = app.shell()
      .sidecar("yt-dlp")?
      .args(args)
      .spawn()?;

  while let Some(event) = rx.recv().await {
      match event {
          CommandEvent::Stdout(bytes) => { /* parse linha */ }
          CommandEvent::Stderr(bytes) => { /* guardar últimas N linhas */ }
          CommandEvent::Terminated(status) => { /* done ou error */ }
          _ => {}
      }
  }
  ```
- Como o spawn é feito **no Rust**, o front não precisa de permissão de shell nas capabilities — mantenha o front sem acesso ao shell.
- Caminho do ffmpeg para `--ffmpeg-location`: no app empacotado os sidecars ficam ao lado do executável principal (sem o triple no nome) → resolver com `std::env::current_exe()?.parent()`.

### Atualização do yt-dlp
O binário empacotado envelhece rápido (YouTube quebra o yt-dlp com frequência). Estratégia:
1. Na primeira execução, copiar o sidecar para a pasta de dados do app (`app.path().app_data_dir()`).
2. Executar sempre essa cópia.
3. Botão "Atualizar yt-dlp" roda `yt-dlp -U` nessa cópia.

---

## 10. Parser de progresso (`progress.rs`)

- Linhas que começam com `dl|` → progresso: `dl|<percent>|<speed>|<eta>`
  - `percent` vem como `" 42.1%"` → trim, remover `%`, parse `f32`.
  - `speed`/`eta` podem vir como `"Unknown"` / `"NA"` → manter string.
- Linha que é um caminho de arquivo existente (do `--print after_move:filepath`) → guardar como `filepath` final.
- Demais linhas → log (opcional mostrar na UI em um painel "detalhes").
- Vídeo + áudio separados geram **dois ciclos de 0→100%**. Opcional: detectar `[download] Destination:` e mostrar "Baixando vídeo / Baixando áudio / Mesclando".

---

## 11. Segurança

- **Nunca** montar comando como string de shell. Sempre `Vec<String>` de argumentos.
- O backend aceita **apenas** IDs presentes em `options.rs`; valores de `Select`/`Preset` devem estar na lista de `choices`.
- Campos `Text`: rejeitar valores que comecem com `-` (evita injeção de flags).
- URL: validar que é `http(s)://`.
- Capabilities do Tauri: liberar só o necessário (`core`, `dialog`, `store`, `opener`, eventos). **Sem** shell no front.

---

## 12. Erros comuns para traduzir na UI

| Trecho no stderr | Mensagem amigável |
|---|---|
| `Could not copy Chrome cookie database` / `Failed to decrypt` | "Feche o navegador e tente de novo, ou use cookies do Firefox." |
| `Sign in to confirm you're not a bot` | "O YouTube pediu login. Ative 'Usar cookies do navegador'." |
| `Requested format is not available` | "Formato indisponível para este vídeo. Tente 'Melhor qualidade'." |
| `ffmpeg not found` / `ffprobe` | "ffmpeg não encontrado — reinstale o app." |
| `Video unavailable` / `Private video` | "Vídeo indisponível ou privado." |
| `HTTP Error 403` | "Acesso negado. Tente atualizar o yt-dlp." |

---

## 13. UI — layout

```
┌───────────────────────────────────────────────────────────┐
│ [ Cole a URL aqui ...........................] [Baixar]  │
├───────────────────────────────────────────────────────────┤
│ Pasta: C:\Users\...\Videos                    [Alterar]  │
├──────────────────────────────┬────────────────────────────┤
│ ▸ Formato                    │ Fila                       │
│   Qualidade: [Melhor     ▾]  │ ┌────────────────────────┐ │
│   Container: [mp4        ▾]  │ │ Título do vídeo        │ │
│ ▸ Autenticação               │ │ ████████░░░ 72%        │ │
│   ☑ Cookies do navegador     │ │ 3.2MiB/s · 00:12   [✕] │ │
│     [firefox ▾]              │ └────────────────────────┘ │
│ ▸ Legendas                   │                            │
│   ☐ Baixar legendas          │                            │
│   ...                        │                            │
├──────────────────────────────┴────────────────────────────┤
│ $ yt-dlp -f "bv*+ba/b" --cookies-from-browser firefox ... │
│                                          [Copiar comando] │
└───────────────────────────────────────────────────────────┘
```

- Grupos recolhíveis.
- Prévia do comando atualiza em tempo real (`preview_command`).
- Opções selecionadas e pasta persistem entre sessões.
- Tema claro/escuro seguindo o sistema.

---

## 14. Fases de implementação

### Fase 1 — Esqueleto
- [ ] `npm create tauri-app@latest` (Svelte + TS)
- [ ] Adicionar plugins: shell, store, dialog, opener
- [ ] Sidecars `yt-dlp` e `ffmpeg` configurados
- [ ] Comando `start_download` baixando uma URL com args fixos
- **Aceite:** clicar em "Baixar" salva o vídeo na pasta de Downloads.

### Fase 2 — Progresso
- [ ] `--progress-template` + parser
- [ ] Eventos `download://*`
- [ ] `QueueItem` com barra, velocidade e ETA
- **Aceite:** barra anda em tempo real e termina com o caminho do arquivo.

### Fase 3 — Opções dinâmicas
- [ ] `options.rs` com a tabela da seção 5
- [ ] `get_options` + `OptionsPanel` gerado automaticamente
- [ ] `args.rs` + validação (seção 11)
- [ ] `preview_command` + `CommandPreview`
- **Aceite:** marcar/desmarcar opções altera a prévia e o download respeita as opções.

### Fase 4 — Fila e configurações
- [ ] Fila com limite de concorrência
- [ ] Cancelar download
- [ ] Persistir settings (pasta, opções marcadas)
- [ ] Seletor de pasta
- **Aceite:** 5 URLs na fila, 2 baixando por vez, cancelar funciona, reabrir o app mantém as escolhas.

### Fase 5 — Polimento e distribuição
- [ ] Cópia do yt-dlp para app data + botão "Atualizar yt-dlp"
- [ ] Tradução dos erros comuns (seção 12)
- [ ] Botão "Abrir pasta" ao concluir
- [ ] `npm run tauri build` → instalador
- **Aceite:** instalador < 120 MB (ffmpeg é o grosso), app abre em < 1 s.

---

## 15. Notas para o Claude Code

- Seguir a estrutura da seção 3; manter cada módulo Rust pequeno e focado.
- Tipos em `src/lib/types.ts` devem espelhar os structs Rust (serde `camelCase` ou `snake_case` — escolher um e manter).
- Não adicionar dependências pesadas no front (sem React, sem UI kits grandes).
- Escrever testes unitários para `args.rs` (mapeamento opções → args, validação) e `progress.rs` (parse de linhas reais).
- Implementar uma fase por vez e validar o critério de aceite antes de seguir.
