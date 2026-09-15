use std::fs;

use pulldown_cmark::{CowStr, Event, HeadingLevel, Tag, TagEnd};

struct HeadingInfo {
    level: HeadingLevel,
    id: String,
    text: String,
}

fn convert_markdown_to_html(markdown: &str) -> String {
    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_TABLES);

    let mut events: Vec<Event> = pulldown_cmark::Parser::new_ext(markdown, options).collect();

    // 目次の生成
    let mut headings = Vec::new();

    let mut current_text = String::new();

    let mut in_heading: Option<HeadingLevel> = None;

    let mut count = 0;

    for event in events.iter_mut() {
        match event {
            Event::Start(Tag::Heading { level, id, classes, attrs }) if *level <= HeadingLevel::H3 => {
                count += 1;
                *id = Some(CowStr::from(format!("h-{count}")));
                in_heading = Some(*level);
                current_text.clear();
            },
            Event::Text(text) if in_heading.is_some() => current_text.push_str(text),
            Event::End(TagEnd::Heading(level)) if in_heading == Some(*level) => {
                headings.push(HeadingInfo {
                    level: *level,
                    id: format!("h-{count}"),
                    text: current_text.clone(),
                });
                in_heading = None;
            },
            _ => {},
        }
    }

    let mut toc = String::from("<nav class=\"toc\"><ul>");
    for h in &headings {
        toc.push_str(&format!(
            "<li class=\"toc-{:?}\"><a href=\"#{}\">{}</a></li>",
            h.level, h.id, h.text
        ));
    }
    toc.push_str("</ul></nav>");

    let mut buffer = String::new();
    pulldown_cmark::html::push_html(&mut buffer, events.into_iter());

    format!("{toc}<main class=\"main\">{buffer}</main>")
}

fn build_page(md_path: &str, out_path: &str, css_hrefs: &[&str]) -> std::io::Result<()> {
    let content = fs::read_to_string(md_path)?;

    let res = convert_markdown_to_html(&content);

    let css_links: String = css_hrefs
        .iter()
        .map(|href| format!("<link rel=\"stylesheet\" href=\"{href}\">"))
        .collect();

    let html = format!(
        "<html>
            <head>
                <meta charset=\"utf-8\">
                <meta lang=\"ja\">
                <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
                <meta name=\"robots\" content=\"noindex, nofollow\">
                <link rel=\"preconnect\" href=\"https://fonts.googleapis.com\">
                <link rel=\"preconnect\" href=\"https://fonts.gstatic.com\" crossorigin>
                <link href=\"https://fonts.googleapis.com/css2?family=Noto+Sans+JP:wght@400;700&family=Roboto:wght@400;700&display=swap\" rel=\"stylesheet\">
                {css_links}
            </head>
            <body>
                <div class=\"wrapper\">
                    {res}
                </div>
                <footer class=\"footer\">
                    <a
                        class=\"footer-github\"
                        href=\"https://github.com/kento-yoshidu/mini_ssg\"
                        target=\"_blank\"
                        rel=\"noopener noreferrer\"
                        aria-label=\"GitHub\"
                    >
                        <svg viewBox=\"0 0 16 16\" width=\"48\" height=\"48\" fill=\"currentColor\" aria-hidden=\"true\">
                            <path d=\"M8 0c4.42 0 8 3.58 8 8a8.013 8.013 0 0 1-5.45 7.59c-.4.08-.55-.17-.55-.38 0-.27.01-1.13.01-2.2 0-.75-.25-1.23-.54-1.48 1.78-.2 3.65-.88 3.65-3.95 0-.88-.31-1.59-.82-2.15.08-.2.36-1.02-.08-2.12 0 0-.67-.22-2.2.82-.64-.18-1.32-.27-2-.27-.68 0-1.36.09-2 .27-1.53-1.03-2.2-.82-2.2-.82-.44 1.1-.16 1.92-.08 2.12-.51.56-.82 1.28-.82 2.15 0 3.06 1.86 3.75 3.64 3.95-.23.2-.44.55-.51 1.07-.46.21-1.61.55-2.33-.66-.15-.24-.6-.83-1.23-.82-.67.01-.27.38.01.53.34.19.73.9.82 1.13.16.45.68 1.31 2.69.94 0 .67.01 1.3.01 1.49 0 .21-.15.45-.55.38A7.995 7.995 0 0 1 0 8c0-4.42 3.58-8 8-8Z\"></path>
                        </svg>
                    </a>
                </footer>
            </body>
        </html>"
    );

    if let Some(parent) = std::path::Path::new(out_path).parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(out_path, html)?;

    Ok(())
}

fn main() -> std::io::Result<()> {
    if fs::exists("dist")? {
        fs::remove_dir_all("dist")?;
    }

    fs::create_dir_all("dist")?;

    build_page("content/index.md", "dist/index.html", &["style.css"])?;

    let mut dirs: Vec<String> = Vec::new();

    for entry in fs::read_dir("content")? {
        let entry = entry?;

        let path = entry.path();

        if path.is_dir() && path.join("index.md").exists() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                dirs.push(name.to_string());
            }
        }
    }

    dirs.sort();

    for dir in dirs.iter() {
        let md = format!("content/{dir}/index.md");
        let out = format!("dist/{dir}/index.html");
        build_page(&md, &out, &["../style.css", "../page.css"])?;
    }

    fs::copy("static/style.css", "dist/style.css")?;
    fs::copy("static/page.css", "dist/page.css")?;

    Ok(())
}
