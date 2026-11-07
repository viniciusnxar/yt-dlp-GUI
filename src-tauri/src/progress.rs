// Parser das linhas de saída do yt-dlp e tradução de erros comuns.

#[derive(Debug, PartialEq)]
pub struct Progress {
    pub percent: f32,
    pub speed: String,
    pub eta: String,
}

/// "dl| 42.1%|3.20MiB/s|00:15" -> Progress. Outras linhas -> None.
pub fn parse(line: &str) -> Option<Progress> {
    let mut p = line.trim().strip_prefix("dl|")?.split('|').map(str::trim);
    let percent = p.next()?.trim_end_matches('%').parse().unwrap_or(0.0);
    Some(Progress {
        percent,
        speed: p.next().unwrap_or("").to_string(),
        eta: p.next().unwrap_or("").to_string(),
    })
}

const ERRORS: &[(&[&str], &str)] = &[
    (&["Could not copy Chrome cookie database", "Failed to decrypt"], "Feche o navegador e tente de novo, ou use cookies do Firefox."),
    (&["Sign in to confirm you"], "O YouTube pediu login. Ative 'Cookies do navegador'."),
    (&["Requested format is not available"], "Formato indisponível para este vídeo. Tente 'Melhor'."),
    (&["ffmpeg not found", "ffprobe"], "ffmpeg não encontrado — reinstale o app."),
    (&["Video unavailable", "Private video"], "Vídeo indisponível ou privado."),
    (&["HTTP Error 403"], "Acesso negado. Tente atualizar o yt-dlp."),
];

/// Mensagem amigável a partir das últimas linhas do stderr.
pub fn friendly_error(stderr_tail: &[String]) -> String {
    let all = stderr_tail.join("\n");
    ERRORS
        .iter()
        .find(|(needles, _)| needles.iter().any(|n| all.contains(n)))
        .map(|(_, msg)| msg.to_string())
        .or_else(|| stderr_tail.iter().rev().find(|l| l.contains("ERROR")).cloned())
        .unwrap_or_else(|| "Falha no download.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_real_lines() {
        assert_eq!(
            parse("dl| 42.1%|   3.20MiB/s|00:15"),
            Some(Progress { percent: 42.1, speed: "3.20MiB/s".into(), eta: "00:15".into() })
        );
        assert_eq!(
            parse("dl|100.0%|Unknown B/s|NA"),
            Some(Progress { percent: 100.0, speed: "Unknown B/s".into(), eta: "NA".into() })
        );
        assert_eq!(parse("dl|  N/A%|NA|NA").unwrap().percent, 0.0);
        assert_eq!(parse("C:\\Videos\\x [abc].mp4"), None);
        assert_eq!(parse("[youtube] abc: Downloading webpage"), None);
    }

    #[test]
    fn translates_errors() {
        let t = |s: &str| friendly_error(&[s.to_string()]);
        assert_eq!(t("ERROR: [youtube] x: Sign in to confirm you're not a bot"), "O YouTube pediu login. Ative 'Cookies do navegador'.");
        assert_eq!(t("ERROR: unable to download video data: HTTP Error 403: Forbidden"), "Acesso negado. Tente atualizar o yt-dlp.");
        assert_eq!(t("ERROR: something new"), "ERROR: something new");
        assert_eq!(t("WARNING: meh"), "Falha no download.");
    }
}
