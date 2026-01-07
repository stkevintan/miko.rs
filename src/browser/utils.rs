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
