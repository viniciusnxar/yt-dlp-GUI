// Fila de jobs + spawn/kill do yt-dlp + leitura de stdout/stderr.
use crate::progress;
use serde::Serialize;
use serde_json::json;
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::{CommandChild, CommandEvent};

const MAX_CONCURRENT: usize = 2;
const STDERR_TAIL: usize = 20;

#[derive(Serialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Queued,
    Running,
    Done,
    Error,
    Cancelled,
}

#[derive(Serialize, Clone)]
pub struct JobInfo {
    pub id: String,
    pub url: String,
    pub title: Option<String>,
    pub status: Status,
    pub percent: f32,
    pub speed: String,
    pub eta: String,
    pub filepath: Option<String>,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct AppState {
    jobs: Mutex<Vec<JobInfo>>,
    // Lock order: pending -> jobs -> children.
    pending: Mutex<VecDeque<(String, Vec<String>)>>,
    children: Mutex<HashMap<String, CommandChild>>,
    next_id: AtomicU64,
    pub ytdlp: Mutex<PathBuf>,
}

impl AppState {
    fn update(&self, id: &str, f: impl FnOnce(&mut JobInfo)) {
        if let Some(j) = self.jobs.lock().unwrap().iter_mut().find(|j| j.id == id) {
            f(j)
        }
    }
    pub fn snapshot(&self) -> Vec<JobInfo> {
        self.jobs.lock().unwrap().clone()
    }
}

/// Pasta onde ficam os sidecars (ao lado do executável, sem o triple no nome).
pub fn sidecar_dir() -> PathBuf {
    std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)).unwrap_or_default()
}

/// Copia o yt-dlp empacotado para app data (para o `-U` poder sobrescrever) e retorna o caminho a usar.
// shortcut: a cópia nunca é substituída por um yt-dlp empacotado mais novo; o usuário usa "Atualizar yt-dlp".
pub fn ensure_ytdlp(app: &AppHandle) -> PathBuf {
    let name = format!("yt-dlp{}", std::env::consts::EXE_SUFFIX);
    let src = sidecar_dir().join(&name);
    let Ok(dir) = app.path().app_data_dir() else { return src };
    let dst = dir.join(&name);
    if dst.exists() || (std::fs::create_dir_all(&dir).is_ok() && std::fs::copy(&src, &dst).is_ok()) {
        dst
    } else {
        src
    }
}

pub fn ytdlp_command(app: &AppHandle) -> tauri_plugin_shell::process::Command {
    let path = app.state::<AppState>().ytdlp.lock().unwrap().clone();
    app.shell().command(path).env("PYTHONUTF8", "1").env("PYTHONIOENCODING", "utf-8")
}

pub fn enqueue(app: &AppHandle, url: String, args: Vec<String>) -> String {
    let state = app.state::<AppState>();
    let id = format!("job{}", state.next_id.fetch_add(1, Ordering::Relaxed) + 1);
    state.jobs.lock().unwrap().push(JobInfo {
        id: id.clone(),
        url: url.clone(),
        title: None,
        status: Status::Queued,
        percent: 0.0,
        speed: String::new(),
        eta: String::new(),
        filepath: None,
        error: None,
    });
    state.pending.lock().unwrap().push_back((id.clone(), args));
    let _ = app.emit("download://queued", json!({ "id": id, "url": url }));
    pump(app);
    id
}

/// Dispara jobs pendentes enquanto houver vaga.
fn pump(app: &AppHandle) {
    let state = app.state::<AppState>();
    let mut pending = state.pending.lock().unwrap();
    loop {
        let running = state.jobs.lock().unwrap().iter().filter(|j| j.status == Status::Running).count();
        if running >= MAX_CONCURRENT {
            break;
        }
        let Some((id, args)) = pending.pop_front() else { break };
        match ytdlp_command(app).args(args).spawn() {
            Ok((rx, child)) => {
                state.update(&id, |j| j.status = Status::Running);
                state.children.lock().unwrap().insert(id.clone(), child);
                let _ = app.emit("download://started", json!({ "id": id }));
                tauri::async_runtime::spawn(watch(app.clone(), id, rx));
            }
            Err(e) => fail(app, &id, format!("Não foi possível iniciar o yt-dlp: {e}"), vec![]),
        }
    }
}

fn fail(app: &AppHandle, id: &str, message: String, stderr_tail: Vec<String>) {
    app.state::<AppState>().update(id, |j| {
        j.status = Status::Error;
        j.error = Some(message.clone());
    });
    let _ = app.emit("download://error", json!({ "id": id, "message": message, "stderr_tail": stderr_tail }));
}

async fn watch(app: AppHandle, id: String, mut rx: tauri::async_runtime::Receiver<CommandEvent>) {
    let state = app.state::<AppState>();
    let mut tail: VecDeque<String> = VecDeque::new();
    let mut filepath = None;
    let mut code = None;

    while let Some(ev) = rx.recv().await {
        let (bytes, is_stdout) = match ev {
            CommandEvent::Stdout(b) => (b, true),
            CommandEvent::Stderr(b) => (b, false),
            CommandEvent::Error(e) => (e.into_bytes(), false),
            CommandEvent::Terminated(p) => {
                code = p.code;
                continue;
            }
            _ => continue,
        };
        let line = String::from_utf8_lossy(&bytes).trim_end().to_string();
        // Com --print o yt-dlp fica "quiet" e o progresso pode sair no stderr, então checa os dois.
        if let Some(p) = progress::parse(&line) {
            state.update(&id, |j| {
                j.percent = p.percent;
                j.speed = p.speed.clone();
                j.eta = p.eta.clone();
            });
            let _ = app.emit("download://progress", json!({ "id": id, "percent": p.percent, "speed": p.speed, "eta": p.eta }));
        } else if let Some(t) = line.strip_prefix("title|") {
            state.update(&id, |j| j.title = Some(t.to_string()));
            let _ = app.emit("download://title", json!({ "id": id, "title": t }));
        } else if is_stdout && Path::new(&line).is_file() {
            filepath = Some(line);
        } else if !line.is_empty() {
            if tail.len() == STDERR_TAIL {
                tail.pop_front();
            }
            tail.push_back(line);
        }
    }

    state.children.lock().unwrap().remove(&id);
    let cancelled = state.jobs.lock().unwrap().iter().any(|j| j.id == id && j.status == Status::Cancelled);
    if cancelled {
        let _ = app.emit("download://cancelled", json!({ "id": id }));
    } else if code == Some(0) {
        let fp = filepath.unwrap_or_default();
        state.update(&id, |j| {
            j.status = Status::Done;
            j.percent = 100.0;
            j.filepath = Some(fp.clone());
        });
        let _ = app.emit("download://done", json!({ "id": id, "filepath": fp }));
    } else {
        let tail: Vec<String> = tail.into();
        fail(&app, &id, progress::friendly_error(&tail), tail);
    }
    pump(&app);
}

pub fn cancel(app: &AppHandle, id: &str) {
    let state = app.state::<AppState>();
    let was_pending = {
        let mut p = state.pending.lock().unwrap();
        let n = p.len();
        p.retain(|(i, _)| i != id);
        n != p.len()
    };
    state.update(id, |j| {
        if matches!(j.status, Status::Queued | Status::Running) {
            j.status = Status::Cancelled
        }
    });
    if was_pending {
        let _ = app.emit("download://cancelled", json!({ "id": id }));
    } else if let Some(child) = state.children.lock().unwrap().remove(id) {
        kill_tree(child);
    }
}

// O yt-dlp.exe (PyInstaller) roda um processo filho; matar só o pai deixaria o download rodando.
#[cfg(windows)]
fn kill_tree(child: CommandChild) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let ok = std::process::Command::new("taskkill")
        .args(["/PID", &child.pid().to_string(), "/T", "/F"])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .is_ok_and(|s| s.success());
    if !ok {
        let _ = child.kill();
    }
}

#[cfg(not(windows))]
fn kill_tree(child: CommandChild) {
    let _ = child.kill();
}
