use crate::service::Service;
use once_cell::sync::Lazy;
use regex::Regex;

static LYRICS_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\[(\d+):(\d+)\.(\d+)\](.*)$").unwrap());

impl Service {
    pub fn parse_lrc(content: &str) -> (bool, Vec<(Option<i32>, String)>) {
        let mut lines = Vec::new();
        let mut synced = true;

        for row in content.lines() {
            let row = row.trim();
            if row.is_empty() {
                continue;
            }

            let parsed_line = (|| {
                let caps = LYRICS_RE.captures(row)?;
                let min: i32 = caps[1].parse().ok()?;
                let sec: i32 = caps[2].parse().ok()?;
                let ms_str = &caps[3];
                let mut ms: i32 = ms_str.parse().ok()?;
                if ms_str.len() == 2 {
                    ms *= 10;
                }
                let text = caps[4].trim().to_string();
                let start_time = (min * 60 + sec) * 1000 + ms;
                Some((Some(start_time), text))
            })();

            match parsed_line {
                Some(line) => lines.push(line),
                None => {
                    synced = false;
                    lines.push((None, row.to_string()));
                }
            }
        }

        (synced, lines)
    }
}
