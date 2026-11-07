# yt-dlp GUI

Interface gráfica leve para o [yt-dlp](https://github.com/yt-dlp/yt-dlp), feita com **Tauri v2 + Svelte**.
O app não reimplementa nada: ele monta os argumentos, executa o `yt-dlp` e mostra o progresso.
O `yt-dlp`, o `ffmpeg` e o `ffprobe` já vêm dentro do instalador, então quem usa não precisa instalar mais nada.

---

## Sumário

1. [Instalar e usar (usuário final)](#1-instalar-e-usar-usuário-final)
2. [Preparar o ambiente de desenvolvimento](#2-preparar-o-ambiente-de-desenvolvimento)
3. [Baixar os binários (yt-dlp / ffmpeg / ffprobe)](#3-baixar-os-binários-yt-dlp--ffmpeg--ffprobe)
4. [Rodar em modo desenvolvimento](#4-rodar-em-modo-desenvolvimento)
5. [Gerar o instalador (.exe)](#5-gerar-o-instalador-exe)
6. [Testes](#6-testes)
7. [Como o projeto funciona](#7-como-o-projeto-funciona)
8. [Adicionar uma nova opção do yt-dlp](#8-adicionar-uma-nova-opção-do-yt-dlp)
9. [Onde ficam os dados do app](#9-onde-ficam-os-dados-do-app)
10. [Problemas comuns](#10-problemas-comuns)

---

## 1. Instalar e usar (usuário final)

### Instalar

1. Pegue o instalador `yt-dlp GUI_<versão>_x64-setup.exe`. Ele é gerado em
   `src-tauri/target/release/bundle/nsis/`; veja a [seção 5](#5-gerar-o-instalador-exe).
2. Execute-o e siga os passos. Não precisa de permissão de administrador: a instalação é por usuário.
3. Abra **yt-dlp GUI** pelo menu Iniciar.

Para desinstalar, use *Configurações do Windows → Aplicativos → yt-dlp GUI → Desinstalar*.

### Usar

```
┌───────────────────────────────────────────────────────────┐
│ [ Cole a URL aqui ...........................] [Baixar]   │
│ Pasta: C:\Users\...\Downloads                  [Alterar]  │
├──────────────────────────────┬────────────────────────────┤
│ ▸ Formato                    │ Fila                       │
│   Qualidade  [Melhor     ▾]  │  Título do vídeo           │
│   Container  [mp4        ▾]  │  ████████░░░ 72%           │
│ ▸ Autenticação               │  3.2MiB/s · 00:12     [✕]  │
│   Cookies    [firefox    ▾]  │                            │
│ ▸ Legendas ...               │                            │
├──────────────────────────────┴────────────────────────────┤
│ $ yt-dlp -f "bv*+ba/b" ... <URL>  [Copiar comando]        │
│ yt-dlp 2026.10.26                  [Atualizar yt-dlp]     │
└───────────────────────────────────────────────────────────┘
```

1. **Cole a URL** e clique em **Baixar** (ou aperte Enter). Para enfileirar várias de uma vez, cole-as separadas por espaço ou quebra de linha.
2. **Pasta:** clique em **Alterar** para escolher onde salvar. O padrão é a pasta *Downloads*.
3. **Opções** (painel da esquerda, grupos recolhíveis):
   - Select com `—` = opção desligada.
   - Campo de texto vazio = opção desligada.
   - Passe o mouse sobre uma opção para ver a dica.
4. **Fila** (painel da direita):
   - No máximo **2 downloads ao mesmo tempo**; os outros esperam como "Na fila".
   - **✕** cancela o download (ou o remove da fila).
   - **Abrir pasta** aparece quando o download termina.
   - Em caso de erro aparece uma mensagem amigável (ex.: "O YouTube pediu login. Ative 'Cookies do navegador'.").
5. **Prévia do comando** (rodapé): mostra o comando equivalente do yt-dlp. **Copiar comando** copia para usar no terminal.
6. **Atualizar yt-dlp:** o YouTube muda com frequência e o yt-dlp quebra. Se os downloads começarem a falhar (ex.: `HTTP Error 403`), clique aqui primeiro.

A pasta e as opções marcadas são salvas automaticamente e continuam lá quando você reabre o app.

### Opções disponíveis

| Grupo | Opção | Flag do yt-dlp |
|---|---|---|
| Formato | Qualidade (Melhor / Até 1080p / Até 720p / Só áudio MP3 / M4A) | `-f ...` ou `-x --audio-format ...` |
| Formato | Container (mp4, mkv, webm) | `--merge-output-format` |
| Autenticação | Cookies do navegador | `--cookies-from-browser` |
| Legendas | Baixar legendas / automáticas / idiomas / embutir | `--write-subs` `--write-auto-subs` `--sub-langs` `--embed-subs` |
| Metadados | Embutir thumbnail / metadados / capítulos | `--embed-thumbnail` `--embed-metadata` `--embed-chapters` |
| SponsorBlock | Remover segmentos | `--sponsorblock-remove` |
| Playlist | Só o vídeo / itens | `--no-playlist` `--playlist-items` |
| Arquivo | Nome do arquivo / nomes restritos / arquivo de histórico | `-o` `--restrict-filenames` `--download-archive` |
| Rede | Limite de velocidade / fragmentos simultâneos | `-r` `-N` |

> **Dica sobre cookies:** Chrome, Edge e Brave travam o banco de cookies enquanto estão abertos.
> Se der erro, feche o navegador ou use o **Firefox**.

---

## 2. Preparar o ambiente de desenvolvimento

Necessário só para quem vai **modificar ou compilar** o app (testado no Windows 11).

| Ferramenta | Para quê | Instalação (winget) |
|---|---|---|
| **Node.js** 20+ | front-end (Vite/Svelte) | `winget install OpenJS.NodeJS.LTS` |
| **Rust** (rustup, toolchain MSVC) | back-end Tauri | `winget install Rustlang.Rustup` |
| **VS 2022 Build Tools** (C++) | linker MSVC exigido pelo Rust | veja abaixo |
| **WebView2** | renderiza a UI | já vem no Windows 10/11 |

```powershell
winget install --id Microsoft.VisualStudio.2022.BuildTools --override "--quiet --wait --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
winget install --id Rustlang.Rustup
```

Depois de instalar, **abra um novo terminal** para o `cargo` entrar no PATH e confira:

```powershell
node -v
cargo -V
rustc -Vv   # a linha "host" mostra o target triple, ex.: x86_64-pc-windows-msvc
```

Instale as dependências do projeto:

```powershell
npm install
```

Para Linux/macOS, veja os pré-requisitos em <https://tauri.app/start/prerequisites/>.

---

## 3. Baixar os binários (yt-dlp / ffmpeg / ffprobe)

Os binários **não estão no git** (são cerca de 360 MB; veja o `.gitignore`). Baixe-os antes de rodar ou compilar.
Eles ficam em `src-tauri/binaries/`, com o **target triple** no nome (exigência do Tauri para sidecars):

```
src-tauri/binaries/
├── yt-dlp-x86_64-pc-windows-msvc.exe
├── ffmpeg-x86_64-pc-windows-msvc.exe
└── ffprobe-x86_64-pc-windows-msvc.exe
```

### PowerShell (Windows)

Execute na raiz do projeto:

```powershell
$T = "x86_64-pc-windows-msvc"
$B = "src-tauri\binaries"
New-Item -ItemType Directory -Force $B | Out-Null

# yt-dlp
Invoke-WebRequest "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe" -OutFile "$B\yt-dlp-$T.exe"

# ffmpeg + ffprobe (builds mantidas pelo time do yt-dlp)
$tmp = New-Item -ItemType Directory -Force "$env:TEMP\ffdl"
Invoke-WebRequest "https://github.com/yt-dlp/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip" -OutFile "$tmp\ff.zip"
Expand-Archive "$tmp\ff.zip" -DestinationPath $tmp -Force
$bin = Get-ChildItem $tmp -Recurse -Filter ffmpeg.exe | Select-Object -First 1
Copy-Item $bin.FullName "$B\ffmpeg-$T.exe"
Copy-Item (Join-Path $bin.DirectoryName "ffprobe.exe") "$B\ffprobe-$T.exe"
Remove-Item $tmp -Recurse -Force

& "$B\yt-dlp-$T.exe" --version   # confere
```

### Git Bash

```bash
T=x86_64-pc-windows-msvc; B=src-tauri/binaries; mkdir -p $B
curl -L -o $B/yt-dlp-$T.exe https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe
D=$(mktemp -d)
curl -L -o $D/ff.zip https://github.com/yt-dlp/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip
unzip -q -j $D/ff.zip '*/bin/ffmpeg.exe' '*/bin/ffprobe.exe' -d $D
mv $D/ffmpeg.exe $B/ffmpeg-$T.exe; mv $D/ffprobe.exe $B/ffprobe-$T.exe; rm -rf $D
```

**Outra plataforma?** Troque o triple pelo que aparece em `rustc -Vv` → `host`, por exemplo
`x86_64-unknown-linux-gnu`, `aarch64-apple-darwin` ou `x86_64-apple-darwin`, e baixe os binários
dessa plataforma (sem `.exe`).

**Atualizar os binários embutidos** (para uma nova versão do instalador): rode os comandos acima de novo
(eles sobrescrevem os arquivos) e gere o instalador outra vez.

---

## 4. Rodar em modo desenvolvimento

```powershell
npm run tauri dev
```

- Abre a janela do app com hot-reload do front (Vite na porta **1420**).
- Mudanças no Rust (`src-tauri/src`) recompilam e reiniciam o app sozinhas.
- A primeira compilação do Rust leva alguns minutos; as próximas levam segundos.
- O Tauri copia os sidecars para `src-tauri/target/debug/` (sem o triple no nome).

> A porta 1420 precisa estar livre. Se estiver ocupada, feche o outro `tauri dev`/`vite`.

---

## 5. Gerar o instalador (.exe)

```powershell
npm run tauri build
```

Saída:

```
src-tauri/target/release/yt-gui.exe                                   # executável solto
src-tauri/target/release/bundle/nsis/yt-dlp GUI_<versão>_x64-setup.exe  # instalador (~117 MB)
```

- O alvo do instalador é **NSIS** (`bundle.targets` em `src-tauri/tauri.conf.json`). Para gerar também um `.msi`, adicione `"msi"` à lista.
- A **versão** do app fica em `src-tauri/tauri.conf.json` (`"version"`); mantenha `package.json` e `src-tauri/Cargo.toml` iguais.
- Na primeira vez o Tauri baixa o NSIS automaticamente.
- O tamanho vem quase todo do ffmpeg e do ffprobe.

> ⚠️ O executável **não é assinado**. Na primeira execução o Windows SmartScreen pode avisar;
> clique em *Mais informações → Executar assim mesmo*.

---

## 6. Testes

```powershell
cd src-tauri
cargo test        # testes do Rust: args.rs (opções → args, validação) e progress.rs (parser, erros)
cd ..
npm run check     # checagem de tipos do Svelte/TypeScript
```

---

## 7. Como o projeto funciona

```
[UI Svelte] ──invoke("start_download", {url, selected, folder})──▶ [commands.rs]
                                                                       │ valida (args.rs)
                                                                       ▼
                                                                  [runner.rs] fila (máx. 2)
                                                                       │ spawn yt-dlp
                                                                       ▼
                                                     stdout/stderr linha a linha
                                                                       │ progress.rs
[UI] ◀── eventos download://queued|started|progress|title|done|error|cancelled
```

### Estrutura

```
src/                          Front-end (Svelte 5 + TypeScript)
├── App.svelte                Tela inteira: URL, pasta, opções, fila, prévia
├── components/
│   ├── OptionControl.svelte  Renderiza 1 opção (toggle/select/preset/text/number/path)
│   └── QueueItem.svelte      Item da fila: barra, velocidade, ETA, cancelar, abrir pasta
├── lib/
│   ├── api.ts                Wrappers tipados de invoke()/listen()
│   ├── types.ts              Tipos espelhando os structs Rust (snake_case)
│   └── stores.ts             Persistência (pasta + opções) via tauri-plugin-store
└── routes/+page.svelte       Só carrega o App.svelte (template SvelteKit em modo SPA)

src-tauri/
├── binaries/                 Sidecars (não versionados; veja a seção 3)
├── capabilities/default.json Permissões do front (sem shell!)
├── tauri.conf.json           Janela, bundle, externalBin
└── src/
    ├── lib.rs                Bootstrap: plugins, estado, comandos
    ├── commands.rs           Comandos expostos ao front
    ├── options.rs            Registro de opções (fonte única da verdade)
    ├── args.rs               Opções → Vec<String> + validação + prévia
    ├── runner.rs             Fila, spawn/cancelamento do yt-dlp, eventos
    └── progress.rs           Parser de progresso + tradução de erros
```

### Comandos Tauri

| Comando | Descrição |
|---|---|
| `get_options` | Registro de opções usado para montar a UI |
| `preview_command(selected)` | Comando legível para a prévia |
| `start_download(url, selected, folder)` | Valida, enfileira e retorna o ID do job |
| `cancel_download(jobId)` | Cancela (mata a árvore de processos no Windows) |
| `get_queue` | Estado atual da fila |
| `default_folder` | Pasta Downloads do sistema |
| `update_ytdlp` / `get_ytdlp_version` | `yt-dlp -U` / `yt-dlp --version` |

### Flags fixas (sempre adicionadas pelo backend)

```
--newline --progress --progress-template "download:dl|%(progress._percent_str)s|..."
--print "before_dl:title|%(title)s" --print after_move:filepath
--ffmpeg-location <pasta do executável> -P <pasta de destino> -- <URL>
```

### Segurança

- O comando **nunca** é montado como string de shell: os argumentos são passados como `Vec<String>`.
- O backend só aceita IDs de opção que existem em `options.rs`, e valores de select/preset precisam estar na lista de escolhas.
- Campos de texto/caminho que começam com `-` são rejeitados (evita injeção de flags).
- A URL precisa começar com `http://` ou `https://`, e vem depois de `--`.
- O front **não tem** permissão de shell; só o Rust executa processos.

---

## 8. Adicionar uma nova opção do yt-dlp

Basta **uma linha** em `src-tauri/src/options.rs`, dentro de `all()`. A UI é gerada sozinha.

```rust
// Toggle (flag sem valor)
opt("Metadados", "write_description", "--write-description", "Salvar descrição", "Salva .description", Toggle),

// Select (flag + valor de uma lista)
opt("Formato", "audio_quality", "--audio-quality", "Qualidade do áudio", "0 = melhor", select(&["0", "5", "9"])),

// Texto livre
opt("Rede", "proxy", "--proxy", "Proxy", "Ex.: socks5://host:porta", text("socks5://...")),

// Número com limites
opt("Rede", "retries", "--retries", "Tentativas", "0–20", Number { min: 0, max: 20 }),

// Caminho (abre diálogo de arquivo)
opt("Autenticação", "cookies_file", "--cookies", "Arquivo de cookies", "cookies.txt", Path),
```

Para uma escolha que gera **vários** argumentos, use `Preset` (veja `quality` no mesmo arquivo).
Depois rode `cargo test` para conferir que nada quebrou.

---

## 9. Onde ficam os dados do app

Pasta: `%APPDATA%\com.vini.ytdlpgui\`

| Arquivo | Conteúdo |
|---|---|
| `settings.json` | Pasta de destino e opções marcadas |
| `yt-dlp.exe` | Cópia do yt-dlp que o app realmente executa (é a que o botão **Atualizar yt-dlp** atualiza) |

- **Resetar configurações:** apague o `settings.json`.
- **Voltar ao yt-dlp que veio no instalador:** apague o `yt-dlp.exe` desta pasta. Ele é recopiado na próxima abertura.
  Faça isso também depois de instalar uma versão nova do app, porque a cópia antiga **não** é substituída automaticamente.

---

## 10. Problemas comuns

| Sintoma | Solução |
|---|---|
| `Sign in to confirm you're not a bot` | Ative **Cookies do navegador** e faça login no YouTube nesse navegador. |
| Erro de cookies do Chrome/Edge/Brave | Feche o navegador por completo, ou use o Firefox. |
| `HTTP Error 403` / downloads falhando de repente | Clique em **Atualizar yt-dlp**. |
| `Requested format is not available` | Use a qualidade **Melhor**. |
| Aviso "No supported JavaScript runtime" (YouTube) | O yt-dlp recomenda ter o [deno](https://deno.com) instalado para extrair alguns formatos do YouTube: `winget install DenoLand.Deno`. |
| `cargo` não encontrado | Abra um novo terminal após instalar o rustup, ou adicione `%USERPROFILE%\.cargo\bin` ao PATH. |
| `link.exe not found` ao compilar | Falta o VS Build Tools com C++ (seção 2). |
| Erro de sidecar/`externalBin` ao compilar | Faltam os binários em `src-tauri/binaries/` com o triple correto (seção 3). |
| `Port 1420 is already in use` | Feche o outro `npm run tauri dev`. |
