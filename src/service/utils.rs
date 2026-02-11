use once_cell::sync::Lazy;
use regex::Regex;

static LYRICS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\[(\d+):(\d+)\.(\d+)\](.*)$").unwrap());

pub fn strip_articles(name: &str, articles: &[&str]) -> String {
    let upper_name = name.to_uppercase();
    for article in articles {
        let prefix = format!("{} ", article.to_uppercase());
        if upper_name.starts_with(&prefix) {
            return name[prefix.len()..].to_string();
        }
    }
    name.to_string()
}

pub fn create_indexed_list<T, F>(
    items: Vec<T>,
    ignored_articles: &str,
    get_name: F,
) -> Vec<(String, Vec<T>)>
where
    F: Fn(&T) -> &str,
{
    let articles: Vec<&str> = ignored_articles.split_whitespace().collect();
    let mut index_map: std::collections::BTreeMap<String, Vec<T>> =
        std::collections::BTreeMap::new();

    for item in items {
        let name = get_name(&item);
        if name.is_empty() {
            continue;
        }

        let sort_name = strip_articles(name, &articles);
        let first_char = sort_name
            .chars()
            .next()
            .unwrap_or(' ')
            .to_uppercase()
            .to_string();

        index_map.entry(first_char).or_default().push(item);
    }

    let mut result: Vec<(String, Vec<T>)> = index_map.into_iter().collect();
    for (_, group) in &mut result {
        group.sort_by(|a, b| get_name(a).to_lowercase().cmp(&get_name(b).to_lowercase()));
    }

    result
}

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
