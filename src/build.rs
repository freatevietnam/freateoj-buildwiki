use anyhow::Result;
use chrono::Local;
use pulldown_cmark::{html, Options, Parser};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::db::{Page, Section};
use crate::svg;

const CDN_HIGHLIGHT_CSS: &str = "https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11.11.1/build/styles/github.min.css";
const CDN_HIGHLIGHT_JS: &str = "https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11.11.1/build/highlight.min.js";
const CDN_MATHJAX: &str = "https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js";
const CDN_HTMX: &str = "https://cdn.jsdelivr.net/npm/htmx.org@2.0.10/dist/htmx.min.js";

pub fn render_markdown(text: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(text, options);

    // Pre-process: add target="_blank" to external links
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // Post-process: add target="_blank" to http links
    html_output = html_output
        .replace("<a href=\"http", "<a target=\"_blank\" rel=\"noopener noreferrer\" href=\"http");

    html_output
}

fn esc(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn json_ld_article(
    title: &str,
    desc: &str,
    url: &str,
    image: &str,
    author: &str,
    published: &str,
) -> String {
    format!(
        r#"{{"@context":"https://schema.org","@graph":[{{"@type":"Article","headline":"{}","description":"{}","url":"{}","image":"{}","author":{{"@type":"Organization","name":"{}"}},"datePublished":"{}","dateModified":"{}"}}]}}"#,
        esc(title), esc(desc), esc(url), esc(image), esc(author), published, published
    )
}

fn json_ld_website(name: &str, url: &str, desc: &str, author: &str) -> String {
    format!(
        r#"{{"@context":"https://schema.org","@type":"WebSite","name":"{}","url":"{}/","description":"{}","author":{{"@type":"Organization","name":"{}"}}}}"#,
        esc(name), esc(url), esc(desc), esc(author)
    )
}

fn head(
    title: &str,
    site_name: &str,
    desc: &str,
    keywords: &str,
    canonical: &str,
    json_ld: &str,
    locale: &str,
    site_author: &str,
    og_image: &str,
    og_w: &str,
    og_h: &str,
    tw_site: &str,
    tw_card: &str,
    robots: &str,
) -> String {
    let kw = if !keywords.is_empty() {
        format!(r#"<meta name="keywords" content="{}">"#, esc(keywords))
    } else {
        String::new()
    };

    format!(
        r#"<head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0">
<title>{title} - {site_name}</title>
<meta name="title" content="{title} - {site_name}">
<meta name="description" content="{desc}">{kw}
<meta name="author" content="{author}">
<meta name="robots" content="{robots}">
<link rel="canonical" href="{canonical}">
<meta property="og:type" content="article">
<meta property="og:title" content="{title} - {site_name}">
<meta property="og:description" content="{desc}">
<meta property="og:url" content="{canonical}">
<meta property="og:site_name" content="{site_name}">
<meta property="og:locale" content="{locale}">
<meta property="og:image" content="{og_image}">
<meta property="og:image:width" content="{og_w}">
<meta property="og:image:height" content="{og_h}">
<meta name="twitter:card" content="{tw_card}">
<meta name="twitter:site" content="{tw_site}">
<meta name="twitter:title" content="{title} - {site_name}">
<meta name="twitter:description" content="{desc}">
<meta name="twitter:image" content="{og_image}">
<script type="application/ld+json">{json_ld}</script>
<link rel="icon" href="data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 100 100%22><text y=%22.9em%22 font-size=%2290%22>📖</text></svg>">
<link rel="stylesheet" href="wiki.css">
<link rel="stylesheet" href="{CDN_HIGHLIGHT_CSS}">
<script>MathJax={{ tex: {{ inlineMath: [['$','$']], displayMath: [['$$','$$']] }} }};</script>
<script src="{CDN_MATHJAX}" defer></script>
<script src="{CDN_HTMX}"></script></head>"#,
        title = esc(title),
        site_name = esc(site_name),
        desc = esc(desc),
        author = esc(site_author),
        robots = esc(robots),
        canonical = esc(canonical),
        locale = esc(locale),
        og_image = esc(og_image),
        og_w = esc(og_w),
        og_h = esc(og_h),
        tw_site = esc(tw_site),
        tw_card = esc(tw_card),
        json_ld = json_ld,
        CDN_HIGHLIGHT_CSS = CDN_HIGHLIGHT_CSS,
        CDN_MATHJAX = CDN_MATHJAX,
        CDN_HTMX = CDN_HTMX,
    )
}

fn sidebar_html(
    sections: &[Section],
    pages: &[Page],
    site_name: &str,
    current_slug: &str,
) -> String {
    let mut parts = vec![
        format!(
            r#"<div class="sidebar-header"><a href="index.html"><h2>{}</h2></a><p>Knowledge Base</p></div>"#,
            esc(site_name)
        ),
        r#"<div class="sidebar-search"><input type="text" id="wikiSearch" placeholder="Search pages..."></div>"#.to_string(),
        "<nav><ul>".to_string(),
    ];

    for section in sections {
        parts.push(format!(
            r#"<li><span class="section-title">{}</span><ul>"#,
            esc(&section.title)
        ));
        for page in pages {
            if page.section_id != section.id {
                continue;
            }
            let active = if page.slug == current_slug {
                " class=\"active\""
            } else {
                ""
            };
            parts.push(format!(
                r#"<li><a href="{}.html"{}>{}</a></li>"#,
                esc(&page.slug),
                active,
                esc(&page.title)
            ));
        }
        parts.push("</ul></li>".to_string());
    }

    parts.push("</ul></nav>".to_string());
    parts.push(r#"<div class="search-empty" id="wikiSearchEmpty">No matching pages</div>"#.to_string());

    parts.join("\n")
}

fn page_template() -> &'static str {
    r#"<!DOCTYPE html>
<html lang="LANG">
HEAD
<body hx-boost="true">
<button class="menu-toggle" id="menuToggle">&#9776;</button>
<aside class="sidebar" id="sidebar">SIDEBAR</aside>
<main class="main" id="mainContent">
CONTENT
<footer class="wiki-footer"><p><a href="https://github.com/freatevietnam/freateoj-buildwiki">Edit on GitHub</a> &middot; SITE_NAME</p></footer>
</main>
<script src="CDN_HLJS"></script>
<script>
(function(){
  var t=document.getElementById('menuToggle'),s=document.getElementById('sidebar');
  if(t)t.onclick=function(){s.classList.toggle('open')};
  var searchInput=document.getElementById('wikiSearch');
  var navLinks=document.querySelectorAll('.sidebar nav ul ul li a');
  var searchEmpty=document.getElementById('wikiSearchEmpty');
  if(searchInput){
    searchInput.addEventListener('input',function(){
      var q=this.value.toLowerCase().trim();
      var visible=0;
      navLinks.forEach(function(a){
        var match=!q||a.textContent.toLowerCase().indexOf(q)!==-1;
        a.parentElement.style.display=match?'':'none';
        if(match)visible++;
      });
      document.querySelectorAll('.sidebar nav > ul > li').forEach(function(li){
        var subVisible=li.querySelectorAll('ul li:not([style*="display: none"])');
        li.style.display=subVisible.length?'':'none';
      });
      if(searchEmpty)searchEmpty.style.display=visible===0?'block':'none';
    });
  }
  document.addEventListener('htmx:beforeRequest',function(e){
    var mc=document.getElementById('mainContent');
    if(mc)mc.classList.add('page-exit');
  });
  document.addEventListener('htmx:afterSwap',function(e){
    var mc=document.getElementById('mainContent');
    if(mc)mc.classList.remove('page-exit');
    document.querySelectorAll('code[data-highlighted]').forEach(function(el){delete el.dataset.highlighted});
    if(typeof hljs!=='undefined')hljs.highlightAll();
    if(typeof MathJax!=='undefined'&&MathJax.typesetPromise)MathJax.typesetPromise();
  });
})();
</script></body></html>"#
}

const PAGE_CSS: &str = r#":root {
  --bg: #fafbfe;
  --bg2: #f0f2f8;
  --surface: #ffffff;
  --surface2: #f4f5fa;
  --surface3: #eceef5;
  --text: #1a1d2e;
  --text2: #4a4e6a;
  --text3: #8085a0;
  --accent: #6c5ce7;
  --accent2: #7c6cf7;
  --accent3: #a29bfe;
  --gold: #e17055;
  --gold2: #d63031;
  --border: #e2e5f0;
  --border2: #d0d4e0;
  --success: #00b894;
  --warning: #fdcb6e;
  --danger: #d63031;
  --sidebar-w: 280px;
  --radius: 14px;
  --radius-sm: 8px;
  --shadow: 0 8px 40px rgba(26,29,46,.08);
  --shadow-sm: 0 2px 12px rgba(26,29,46,.06);
}
*, *::before, *::after { margin: 0; padding: 0; box-sizing: border-box; }
html { scroll-behavior: smooth; }
body {
  font-family: 'Inter', 'Segoe UI', system-ui, -apple-system, sans-serif;
  background: var(--bg);
  color: var(--text);
  display: flex;
  min-height: 100vh;
  line-height: 1.7;
}
::selection { background: var(--accent); color: #fff; }
::-webkit-scrollbar { width: 6px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: var(--border2); border-radius: 3px; }
::-webkit-scrollbar-thumb:hover { background: var(--accent); }

.main { animation: fadeSlideIn .35s ease-out; }
@keyframes fadeSlideIn {
  from { opacity: 0; transform: translateY(12px); }
  to { opacity: 1; transform: translateY(0); }
}
@keyframes fadeSlideOut {
  from { opacity: 1; transform: translateY(0); }
  to { opacity: 0; transform: translateY(-8px); }
}
.page-exit { animation: fadeSlideOut .2s ease-in forwards; }

.menu-toggle {
  display: none;
  position: fixed;
  top: 16px;
  left: 16px;
  z-index: 100;
  background: var(--surface);
  color: var(--text);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 10px 16px;
  font-size: 20px;
  cursor: pointer;
  transition: all .2s;
  box-shadow: var(--shadow-sm);
}
.menu-toggle:hover { background: var(--surface2); border-color: var(--accent); }

@media (max-width: 768px) {
  .menu-toggle { display: block; }
  .sidebar {
    transform: translateX(-100%);
    position: fixed;
    z-index: 99;
    height: 100vh;
    transition: transform .3s cubic-bezier(.4,0,.2,1);
  }
  .sidebar.open { transform: translateX(0); }
  .main { margin-left: 0 !important; padding: 48px 20px; }
}

.sidebar {
  width: var(--sidebar-w);
  background: var(--surface);
  border-right: 1px solid var(--border);
  overflow-y: auto;
  flex-shrink: 0;
  height: 100vh;
  position: sticky;
  top: 0;
}
.sidebar-header {
  padding: 28px 22px 20px;
  border-bottom: 1px solid var(--border);
}
.sidebar-header h2 {
  font-size: 18px;
  background: linear-gradient(135deg, var(--accent), var(--gold));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  font-weight: 800;
  letter-spacing: -0.3px;
}
.sidebar-header p {
  font-size: 11px;
  color: var(--text3);
  margin-top: 4px;
  text-transform: uppercase;
  letter-spacing: 1.5px;
}

.sidebar-search {
  padding: 12px 16px 8px;
  position: relative;
}
.sidebar-search input {
  width: 100%;
  padding: 10px 14px 10px 36px;
  background: var(--surface2);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text);
  font-size: 13px;
  outline: none;
  transition: border-color .2s, box-shadow .2s;
}
.sidebar-search input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px rgba(108,92,231,.12);
}
.sidebar-search input::placeholder { color: var(--text3); }
.sidebar-search::before {
  content: "\\1F50D";
  position: absolute;
  left: 28px;
  top: 50%;
  transform: translateY(-50%);
  font-size: 13px;
  pointer-events: none;
  opacity: .4;
}

.sidebar nav { padding: 8px 0; }
.sidebar nav ul { list-style: none; }
.sidebar nav > ul > li { margin-bottom: 2px; }
.section-title {
  display: block;
  padding: 12px 22px 6px;
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 2px;
  color: var(--text3);
  font-weight: 700;
}
.sidebar nav ul ul li a {
  display: block;
  padding: 9px 22px 9px 30px;
  font-size: 13px;
  color: var(--text2);
  text-decoration: none;
  border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
  margin-right: 10px;
  transition: all .15s;
  position: relative;
}
.sidebar nav ul ul li a::before {
  content: "";
  position: absolute;
  left: 22px;
  top: 50%;
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: var(--border2);
  transform: translateY(-50%);
  transition: all .15s;
}
.sidebar nav ul ul li a:hover {
  background: var(--surface2);
  color: var(--text);
}
.sidebar nav ul ul li a:hover::before { background: var(--accent); }
.sidebar nav ul ul li a.active {
  background: linear-gradient(135deg, rgba(108,92,231,.1), rgba(108,92,231,.05));
  color: var(--accent);
  font-weight: 600;
  border: 1px solid rgba(108,92,231,.15);
}
.sidebar nav ul ul li a.active::before {
  background: var(--accent);
  width: 6px;
  height: 6px;
  box-shadow: 0 0 8px var(--accent);
}

.main {
  flex: 1;
  padding: 56px;
  max-width: 900px;
  margin: 0 auto;
}
.main a { color: var(--accent); text-decoration: none; transition: color .15s; }
.main a:hover { color: var(--gold); text-decoration: underline; }

article h1 {
  font-size: 2.4rem;
  margin-bottom: 24px;
  background: linear-gradient(135deg, var(--text), var(--accent));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  font-weight: 800;
  letter-spacing: -1px;
  line-height: 1.2;
}
article h2 {
  font-size: 1.5rem;
  margin: 40px 0 16px;
  color: var(--text);
  font-weight: 700;
  letter-spacing: -0.5px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border);
}
article h3 {
  font-size: 1.15rem;
  margin: 28px 0 12px;
  color: var(--accent);
  font-weight: 600;
}
article p, article li {
  line-height: 1.85;
  margin-bottom: 14px;
  color: var(--text2);
}
article strong { color: var(--text); font-weight: 600; }
article code {
  background: var(--surface2);
  padding: 3px 8px;
  border-radius: var(--radius-sm);
  font-size: .88em;
  font-family: 'JetBrains Mono', 'Fira Code', Consolas, monospace;
  color: var(--accent);
  border: 1px solid var(--border);
}
article pre {
  background: #1e1e2e;
  border: 1px solid #313244;
  padding: 22px;
  border-radius: var(--radius);
  overflow-x: auto;
  margin: 22px 0;
  box-shadow: var(--shadow-sm);
}
article pre code {
  background: none;
  padding: 0;
  color: #cdd6f4;
  font-size: 13px;
  line-height: 1.7;
  border: none;
}
article ul, article ol {
  padding-left: 26px;
  margin-bottom: 16px;
}
article li { margin-bottom: 8px; }
article li::marker { color: var(--accent); }
article img {
  max-width: 100%;
  border-radius: var(--radius);
  box-shadow: var(--shadow);
  margin: 16px 0;
}
article blockquote {
  border-left: 3px solid var(--gold);
  padding: 14px 22px;
  margin: 18px 0;
  background: linear-gradient(135deg, rgba(225,112,85,.05), transparent);
  border-radius: 0 var(--radius) var(--radius) 0;
  color: var(--text2);
  font-style: italic;
}
article hr {
  border: none;
  height: 1px;
  background: linear-gradient(90deg, transparent, var(--border2), transparent);
  margin: 36px 0;
}

.wiki-hero {
  text-align: center;
  padding: 80px 20px 50px;
}
.wiki-hero h1 {
  font-size: 3.2rem;
  background: linear-gradient(135deg, var(--accent), var(--gold), var(--accent));
  background-size: 200% 200%;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  animation: shimmer 4s ease-in-out infinite;
  margin-bottom: 16px;
  font-weight: 800;
  letter-spacing: -1.5px;
}
@keyframes shimmer {
  0%, 100% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
}
.hero-subtitle {
  font-size: 1.15rem;
  color: var(--text2);
  max-width: 640px;
  margin: 0 auto;
  line-height: 1.8;
}
.hero-count {
  margin-top: 20px;
  font-size: .9rem;
  color: var(--text3);
}
.hero-count span {
  color: var(--accent);
  font-weight: 700;
}

.wiki-card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 20px;
  padding: 24px 0;
}
.wiki-card {
  display: block;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 26px;
  text-decoration: none !important;
  transition: all .3s cubic-bezier(.4,0,.2,1);
  position: relative;
  overflow: hidden;
}
.wiki-card::before {
  content: "";
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, var(--accent), var(--gold));
  opacity: 0;
  transition: opacity .3s;
}
.wiki-card:hover {
  border-color: var(--accent);
  transform: translateY(-6px);
  box-shadow: 0 12px 40px rgba(108,92,231,.1);
}
.wiki-card:hover::before { opacity: 1; }
.wiki-card h3 {
  font-size: 1.1rem;
  color: var(--accent);
  margin-bottom: 10px;
  font-weight: 600;
}
.wiki-card p {
  font-size: .88rem;
  color: var(--text3);
  line-height: 1.7;
}

.wiki-footer {
  text-align: center;
  padding: 40px 0 20px;
  border-top: 1px solid var(--border);
  margin-top: 48px;
  font-size: .85rem;
  color: var(--text3);
}
.wiki-footer a { color: var(--accent); }

.search-empty {
  text-align: center;
  padding: 20px;
  color: var(--text3);
  font-size: 13px;
  display: none;
}
"#;

pub fn build(
    settings: &HashMap<String, String>,
    sections: &[Section],
    pages: &[Page],
    output_dir: &str,
) -> Result<()> {
    let out = Path::new(output_dir);
    if out.exists() {
        fs::remove_dir_all(out)?;
    }
    fs::create_dir_all(out)?;

    let site_name = settings.get("SITE_NAME").map(|s| s.as_str()).unwrap_or("Wiki");
    let site_url = settings.get("SITE_URL").map(|s| s.as_str()).unwrap_or("");
    let site_url = site_url.trim_end_matches('/');
    let site_desc = settings.get("SITE_DESCRIPTION").map(|s| s.as_str()).unwrap_or("");
    let site_author = settings.get("SITE_AUTHOR").map(|s| s.as_str()).unwrap_or("");
    let lang = settings.get("DEFAULT_LANG").map(|s| s.as_str()).unwrap_or("en");
    let locale = settings.get("DEFAULT_LOCALE").map(|s| s.as_str()).unwrap_or("");
    let og_image_global = settings.get("OG_IMAGE").map(|s| s.as_str()).unwrap_or("");
    let og_w_global = settings.get("OG_IMAGE_WIDTH").map(|s| s.as_str()).unwrap_or("1200");
    let og_h_global = settings.get("OG_IMAGE_HEIGHT").map(|s| s.as_str()).unwrap_or("630");
    let twitter_site_global = settings.get("TWITTER_SITE").map(|s| s.as_str()).unwrap_or("");
    let twitter_card_global = settings.get("TWITTER_CARD").map(|s| s.as_str()).unwrap_or("summary");
    let robots_global = settings.get("ROBOTS").map(|s| s.as_str()).unwrap_or("index, follow");
    let today = Local::now().format("%Y-%m-%d").to_string();

    // Convert SVG favicon to ICO if available
    let favicon_svg_path = settings.get("FAVICON_SVG").map(|s| s.as_str()).unwrap_or("");
    let _ico_generated = if !favicon_svg_path.is_empty() && Path::new(favicon_svg_path).exists() {
        let ico_path = out.join("favicon.ico");
        match svg::svg_to_ico(favicon_svg_path, &ico_path) {
            Ok(_) => true,
            Err(e) => {
                eprintln!("  [SVG->ICO] Error: {}", e);
                false
            }
        }
    } else {
        false
    };

    let template = page_template();

    // Generate pages
    for page in pages {
        let seo_og_img = if !page.seo_og_image.is_empty() {
            &page.seo_og_image
        } else {
            og_image_global
        };
        let seo_og_w = if !page.seo_og_width.is_empty() {
            &page.seo_og_width
        } else {
            og_w_global
        };
        let seo_og_h = if !page.seo_og_height.is_empty() {
            &page.seo_og_height
        } else {
            og_h_global
        };
        let seo_tw_site = if !page.seo_twitter_site.is_empty() {
            &page.seo_twitter_site
        } else {
            twitter_site_global
        };
        let seo_tw_card = if !page.seo_twitter_card.is_empty() {
            &page.seo_twitter_card
        } else {
            twitter_card_global
        };
        let seo_robots = if !page.seo_robots.is_empty() {
            &page.seo_robots
        } else {
            robots_global
        };

        let canonical = format!("{}/{}.html", site_url, page.slug);
        let content_html = render_markdown(&page.content);
        let ld = json_ld_article(
            &page.title,
            &page.description,
            &canonical,
            seo_og_img,
            site_author,
            &today,
        );
        let h = head(
            &page.title,
            site_name,
            &page.description,
            &page.keywords,
            &canonical,
            &ld,
            locale,
            site_author,
            seo_og_img,
            seo_og_w,
            seo_og_h,
            seo_tw_site,
            seo_tw_card,
            seo_robots,
        );
        let sb = sidebar_html(sections, pages, site_name, &page.slug);
        let body = format!("<article>{}</article>", content_html);

        let html_out = template
            .replace("LANG", lang)
            .replace("HEAD", &h)
            .replace("SIDEBAR", &sb)
            .replace("CONTENT", &body)
            .replace("SITE_NAME", &esc(site_name))
            .replace("CDN_HLJS", CDN_HIGHLIGHT_JS);

        let path = out.join(format!("{}.html", page.slug));
        fs::write(&path, &html_out)?;
        println!("  -> {}.html", page.slug);
    }

    // Generate index
    let mut cards = String::new();
    for page in pages {
        cards.push_str(&format!(
            r#"<a href="{}.html" class="wiki-card"><h3>{}</h3><p>{}</p></a>"#,
            esc(&page.slug),
            esc(&page.title),
            esc(&page.description)
        ));
    }

    let mut hero = format!(
        r#"<div class="wiki-hero"><h1>{}</h1><p class="hero-subtitle">{}</p>"#,
        esc(site_name),
        esc(site_desc)
    );
    if !pages.is_empty() {
        hero.push_str(&format!(
            r#"<p class="hero-count"><span>{}</span> articles</p>"#,
            pages.len()
        ));
    }
    hero.push_str("</div>");

    let grid = if !cards.is_empty() {
        format!("<div class=\"wiki-card-grid\">{}</div>", cards)
    } else {
        String::new()
    };

    let index_body = format!("{}{}", hero, grid);
    let index_canonical = format!("{}/", site_url);
    let index_ld = json_ld_website(site_name, site_url, site_desc, site_author);
    let index_h = head(
        site_name,
        site_name,
        site_desc,
        "",
        &index_canonical,
        &index_ld,
        locale,
        site_author,
        og_image_global,
        og_w_global,
        og_h_global,
        twitter_site_global,
        twitter_card_global,
        robots_global,
    );
    let index_sb = sidebar_html(sections, pages, site_name, "");

    let index_html = template
        .replace("LANG", lang)
        .replace("HEAD", &index_h)
        .replace("SIDEBAR", &index_sb)
        .replace("CONTENT", &index_body)
        .replace("SITE_NAME", &esc(site_name))
        .replace("CDN_HLJS", CDN_HIGHLIGHT_JS);

    fs::write(out.join("index.html"), &index_html)?;
    println!("  -> index.html");

    // Write CSS
    fs::write(out.join("wiki.css"), PAGE_CSS)?;
    println!("  -> wiki.css");

    // Generate sitemap
    let mut sitemap_lines = vec![
        r#"<?xml version="1.0" encoding="UTF-?>"#.to_string(),
        r#"<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#.to_string(),
    ];

    // Index URL
    sitemap_lines.push(format!(
        "<url><loc>{}/{}</loc><lastmod>{}</lastmod><priority>1.0</priority></url>",
        site_url, "", today
    ));

    // Page URLs
    for page in pages {
        sitemap_lines.push(format!(
            "<url><loc>{}/{}.html</loc><lastmod>{}</lastmod><priority>0.8</priority></url>",
            site_url, page.slug, today
        ));
    }

    sitemap_lines.push("</urlset>".to_string());
    fs::write(out.join("sitemap.xml"), sitemap_lines.join("\n"))?;
    println!("  -> sitemap.xml");

    Ok(())
}
