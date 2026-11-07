// Registro de flags: fonte única da verdade. Adicionar uma flag = adicionar uma entrada aqui.
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OptKind {
    Toggle,
    Select { choices: Vec<Choice> },
    Text { placeholder: String },
    Number { min: i64, max: i64 },
    Path,
    Preset { choices: Vec<PresetChoice> },
}

#[derive(Serialize, Clone)]
pub struct Choice {
    pub value: String,
    pub label: String,
}

#[derive(Serialize, Clone)]
pub struct PresetChoice {
    pub id: String,
    pub label: String,
    pub args: Vec<String>,
}

#[derive(Serialize, Clone)]
pub struct OptionDef {
    pub id: String,
    pub flag: String,
    pub label: String,
    pub help: String,
    pub group: String,
    pub kind: OptKind,
    pub default: Option<Value>,
}

fn opt(group: &str, id: &str, flag: &str, label: &str, help: &str, kind: OptKind) -> OptionDef {
    OptionDef {
        id: id.into(),
        flag: flag.into(),
        label: label.into(),
        help: help.into(),
        group: group.into(),
        kind,
        default: None,
    }
}

fn select(values: &[&str]) -> OptKind {
    OptKind::Select {
        choices: values.iter().map(|v| Choice { value: v.to_string(), label: v.to_string() }).collect(),
    }
}

fn text(placeholder: &str) -> OptKind {
    OptKind::Text { placeholder: placeholder.into() }
}

fn preset(id: &str, label: &str, args: &[&str]) -> PresetChoice {
    PresetChoice { id: id.into(), label: label.into(), args: args.iter().map(|s| s.to_string()).collect() }
}

pub fn all() -> Vec<OptionDef> {
    use OptKind::*;
    let mut quality = opt("Formato", "quality", "", "Qualidade", "Formato de vídeo/áudio", Preset {
        choices: vec![
            preset("best", "Melhor", &["-f", "bv*+ba/b"]),
            preset("1080", "Até 1080p", &["-f", "bv*[height<=1080]+ba/b"]),
            preset("720", "Até 720p", &["-f", "bv*[height<=720]+ba/b"]),
            preset("mp3", "Só áudio MP3", &["-x", "--audio-format", "mp3"]),
            preset("m4a", "Só áudio M4A", &["-x", "--audio-format", "m4a"]),
        ],
    });
    quality.default = Some(json!("best"));

    vec![
        quality,
        opt("Formato", "merge_format", "--merge-output-format", "Container", "Formato do arquivo final ao juntar vídeo+áudio", select(&["mp4", "mkv", "webm"])),
        opt("Autenticação", "cookies_browser", "--cookies-from-browser", "Cookies do navegador", "Usa os cookies do navegador (login)", select(&["firefox", "chrome", "edge", "brave", "opera", "vivaldi", "chromium"])),
        opt("Legendas", "write_subs", "--write-subs", "Baixar legendas", "", Toggle),
        opt("Legendas", "write_auto_subs", "--write-auto-subs", "Legendas automáticas", "", Toggle),
        opt("Legendas", "sub_langs", "--sub-langs", "Idiomas", "Ex.: pt.*,en", text("pt.*,en")),
        opt("Legendas", "embed_subs", "--embed-subs", "Embutir legendas", "", Toggle),
        opt("Metadados", "embed_thumbnail", "--embed-thumbnail", "Embutir thumbnail", "Requer ffmpeg", Toggle),
        opt("Metadados", "embed_metadata", "--embed-metadata", "Embutir metadados", "", Toggle),
        opt("Metadados", "embed_chapters", "--embed-chapters", "Embutir capítulos", "", Toggle),
        opt("SponsorBlock", "sponsorblock_remove", "--sponsorblock-remove", "Remover segmentos", "Categorias do SponsorBlock a cortar", select(&["sponsor", "sponsor,selfpromo,intro,outro", "all"])),
        opt("Playlist", "no_playlist", "--no-playlist", "Só o vídeo", "Baixa só o vídeo mesmo se a URL tiver list=", Toggle),
        opt("Playlist", "playlist_items", "--playlist-items", "Itens", "Ex.: 1-5,8", text("1-5,8")),
        opt("Arquivo", "output_template", "-o", "Nome do arquivo", "Template de saída do yt-dlp", text("%(title)s [%(id)s].%(ext)s")),
        opt("Arquivo", "restrict_filenames", "--restrict-filenames", "Nomes restritos", "Só ASCII, sem espaços", Toggle),
        opt("Arquivo", "download_archive", "--download-archive", "Arquivo de histórico", "Pula vídeos já baixados", Path),
        opt("Rede", "limit_rate", "-r", "Limite de velocidade", "Ex.: 5M", text("5M")),
        opt("Rede", "concurrent_fragments", "-N", "Fragmentos simultâneos", "1–16", Number { min: 1, max: 16 }),
    ]
}
