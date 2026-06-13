use std::{io::Write, path::Path};

pub fn write_placeholder_cover(path: &Path, title: &str) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let label = title
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || ch.is_ascii_whitespace())
        .take(64)
        .collect::<String>();
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="640" height="900" viewBox="0 0 640 900">
<rect width="640" height="900" fill="#f4f1ea"/>
<rect x="44" y="44" width="552" height="812" rx="18" fill="#ffffff" stroke="#222222" stroke-width="4"/>
<rect x="92" y="120" width="456" height="18" fill="#c95f45"/>
<text x="92" y="230" font-family="Arial, sans-serif" font-size="42" font-weight="700" fill="#222222">{}</text>
<text x="92" y="790" font-family="Arial, sans-serif" font-size="24" fill="#666666">papercache</text>
</svg>"##,
        escape_xml(&label)
    );
    let mut file = std::fs::File::create(path)?;
    file.write_all(svg.as_bytes())?;
    Ok(())
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
