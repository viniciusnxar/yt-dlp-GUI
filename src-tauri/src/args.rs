// Opções selecionadas -> Vec<String>. Nunca monta string de shell.
use crate::options::{self, OptKind};
use serde_json::Value;
use std::collections::HashMap;

pub type Selected = HashMap<String, Value>;

pub const PROGRESS_TEMPLATE: &str =
    "download:dl|%(progress._percent_str)s|%(progress._speed_str)s|%(progress._eta_str)s";

/// Args das opções do usuário, na ordem do registro. Valor ausente/null/false/"" = desligado.
pub fn user_args(selected: &Selected) -> Result<Vec<String>, String> {
    let defs = options::all();
    if let Some(bad) = selected.keys().find(|k| !defs.iter().any(|d| &d.id == *k)) {
        return Err(format!("Opção desconhecida: {bad}"));
    }
    let mut out = Vec::new();
    for def in &defs {
        let v = match selected.get(&def.id) {
            None | Some(Value::Null) | Some(Value::Bool(false)) => continue,
            Some(Value::String(s)) if s.trim().is_empty() => continue,
            Some(v) => v,
        };
        let bad = || format!("Valor inválido para {}", def.label);
        let s = v.as_str().map(str::trim);
        match &def.kind {
            OptKind::Toggle => {
                if v != &Value::Bool(true) {
                    return Err(bad());
                }
                out.push(def.flag.clone());
            }
            OptKind::Select { choices } => {
                let s = s.filter(|s| choices.iter().any(|c| c.value == *s)).ok_or_else(bad)?;
                out.extend([def.flag.clone(), s.to_string()]);
            }
            OptKind::Preset { choices } => {
                let c = s.and_then(|s| choices.iter().find(|c| c.id == s)).ok_or_else(bad)?;
                out.extend(c.args.iter().cloned());
            }
            OptKind::Text { .. } | OptKind::Path => {
                // Rejeita valores começando com "-" (injeção de flags).
                let s = s.filter(|s| !s.starts_with('-')).ok_or_else(bad)?;
                out.extend([def.flag.clone(), s.to_string()]);
            }
            OptKind::Number { min, max } => {
                let n = v
                    .as_i64()
                    .or_else(|| s.and_then(|s| s.parse().ok()))
                    .filter(|n| (*min..=*max).contains(n))
                    .ok_or_else(bad)?;
                out.extend([def.flag.clone(), n.to_string()]);
            }
        }
    }
    Ok(out)
}

pub fn validate_url(url: &str) -> Result<(), String> {
    let u = url.trim();
    if (u.starts_with("http://") || u.starts_with("https://")) && !u.contains(char::is_whitespace) {
        Ok(())
    } else {
        Err("URL inválida: use http:// ou https://".into())
    }
}

/// Comando completo: opções do usuário + flags fixas + pasta + URL.
pub fn full_args(selected: &Selected, url: &str, folder: &str, ffmpeg_dir: &str) -> Result<Vec<String>, String> {
    validate_url(url)?;
    let mut a = user_args(selected)?;
    a.extend(
        [
            "--newline", "--progress", "--progress-template", PROGRESS_TEMPLATE,
            "--print", "before_dl:title|%(title)s", "--print", "after_move:filepath", "--ffmpeg-location", ffmpeg_dir, "-P", folder,
            "--", url.trim(),
        ]
        .map(String::from),
    );
    Ok(a)
}

/// Comando legível para a prévia.
pub fn preview(selected: &Selected) -> Result<String, String> {
    let quote = |s: &String| {
        if s.is_empty() || s.contains(|c: char| c.is_whitespace() || "\"'*[]<>|&;$%(),".contains(c)) {
            format!("\"{}\"", s.replace('"', "\\\""))
        } else {
            s.clone()
        }
    };
    let mut parts = vec!["yt-dlp".to_string()];
    parts.extend(user_args(selected)?.iter().map(quote));
    parts.push("<URL>".into());
    Ok(parts.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sel(v: Value) -> Selected {
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn maps_options_in_registry_order() {
        let a = user_args(&sel(json!({
            "cookies_browser": "firefox", "quality": "best", "embed_subs": true,
            "write_subs": false, "sub_langs": "", "concurrent_fragments": 4
        })))
        .unwrap();
        assert_eq!(a, ["-f", "bv*+ba/b", "--cookies-from-browser", "firefox", "--embed-subs", "-N", "4"]);
    }

    #[test]
    fn preset_expands_to_many_args() {
        assert_eq!(user_args(&sel(json!({"quality": "mp3"}))).unwrap(), ["-x", "--audio-format", "mp3"]);
    }

    #[test]
    fn rejects_bad_input() {
        for v in [
            json!({"nope": true}),
            json!({"cookies_browser": "safari"}),
            json!({"quality": "4k"}),
            json!({"sub_langs": "--exec calc"}),
            json!({"download_archive": "-x"}),
            json!({"concurrent_fragments": 99}),
            json!({"embed_subs": "yes"}),
        ] {
            assert!(user_args(&sel(v.clone())).is_err(), "{v}");
        }
    }

    #[test]
    fn url_validation_and_full_args() {
        assert!(validate_url("ftp://x").is_err());
        assert!(validate_url("https://a b").is_err());
        let a = full_args(&sel(json!({})), " https://youtu.be/x ", "C:\\d", "C:\\f").unwrap();
        assert_eq!(a.last().unwrap(), "https://youtu.be/x");
        assert_eq!(a[a.len() - 2], "--");
        assert!(a.contains(&"--newline".to_string()));
    }

    #[test]
    fn preview_quotes() {
        let p = preview(&sel(json!({"quality": "720"}))).unwrap();
        assert_eq!(p, "yt-dlp -f \"bv*[height<=720]+ba/b\" <URL>");
    }
}
