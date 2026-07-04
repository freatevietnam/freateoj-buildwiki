#!/usr/bin/env python3
"""
FreateOJ Wiki — single-file wiki builder with GUI, SQLite, and static HTML output.
"""

import os, sys, html, datetime, sqlite3, shutil, http.server, socketserver, mimetypes, threading, json, io, time

try:
    import tkinter as tk
    from tkinter import ttk, messagebox, simpledialog as _sd, filedialog
    import customtkinter as ctk
    _GUI_OK = True
except ImportError:
    _GUI_OK = False
import markdown as md_lib
from markdown.extensions import Extension

try:
    import cairosvg
    from PIL import Image
    _SVG_OK = True
except ImportError:
    _SVG_OK = False

BASE_DIR = os.path.dirname(os.path.abspath(__file__))
DB_PATH = os.path.join(BASE_DIR, "wiki.db")
BUILD_DIR = os.path.join(BASE_DIR, "build")

CDN = {
    "highlight_css": "https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11.11.1/build/styles/github.min.css",
    "highlight_js": "https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11.11.1/build/highlight.min.js",
    "mathjax": "https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js",
    "htmx": "https://cdn.jsdelivr.net/npm/htmx.org@2.0.10/dist/htmx.min.js",
}

MD_EXTENSIONS = ["extra", "sane_lists", "smarty", "toc", "nl2br"]

DEFAULT_SETTINGS = {
    "SITE_NAME": "FreateOJ Wiki",
    "SITE_URL": "https://freatevietnam.github.io/freateoj-wiki",
    "SITE_DESCRIPTION": "Tài liệu và hướng dẫn về nền tảng chấm bài trực tuyến FreateOJ",
    "SITE_AUTHOR": "Freate Vietnam",
    "DEFAULT_LANG": "vi",
    "DEFAULT_LOCALE": "vi_VN",
    "OG_IMAGE": "",
    "OG_IMAGE_WIDTH": "1200",
    "OG_IMAGE_HEIGHT": "630",
    "TWITTER_SITE": "@freatevietnam",
    "TWITTER_CARD": "summary_large_image",
    "ROBOTS": "index, follow",
    "FAVICON_SVG": "",
}

class LinkProcessor(md_lib.treeprocessors.Treeprocessor):
    def run(self, root):
        for elem in root.iter():
            href = elem.get("href", "")
            if elem.tag == "a" and href.startswith("http"):
                elem.set("target", "_blank")
                elem.set("rel", "noopener noreferrer")
        return root

class FreateOJWikiExtension(Extension):
    def extendMarkdown(self, md):
        md.treeprocessors.register(LinkProcessor(md), "freateoj_links", 15)

def render_md(text):
    return md_lib.markdown(text, extensions=[*MD_EXTENSIONS, FreateOJWikiExtension()])


# ── SVG → ICO ────────────────────────────────────────────────────────────────

def svg_to_ico(svg_path, ico_path, sizes=(16, 32, 48, 64, 128, 256)):
    if not _SVG_OK:
        return False
    try:
        png_data = cairosvg.svg2png(url=svg_path)
        img = Image.open(io.BytesIO(png_data))
        imgs = []
        for s in sizes:
            resized = img.resize((s, s), Image.LANCZOS)
            imgs.append(resized)
        imgs[0].save(ico_path, format="ICO", sizes=[(s, s) for s in sizes], append_images=imgs[1:])
        return True
    except Exception as e:
        print(f"  [SVG→ICO] Error: {e}", file=sys.stderr)
        return False


# ── Database ────────────────────────────────────────────────────────────────

def get_conn():
    return sqlite3.connect(DB_PATH)

def init_db():
    conn = get_conn()
    conn.executescript("""
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY, value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS sections (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS pages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            section_id INTEGER NOT NULL,
            slug TEXT NOT NULL UNIQUE,
            title TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            keywords TEXT NOT NULL DEFAULT '',
            content TEXT NOT NULL DEFAULT '',
            sort_order INTEGER NOT NULL DEFAULT 0,
            seo_og_image TEXT NOT NULL DEFAULT '',
            seo_og_width TEXT NOT NULL DEFAULT '',
            seo_og_height TEXT NOT NULL DEFAULT '',
            seo_twitter_site TEXT NOT NULL DEFAULT '',
            seo_twitter_card TEXT NOT NULL DEFAULT '',
            seo_robots TEXT NOT NULL DEFAULT '',
            FOREIGN KEY (section_id) REFERENCES sections(id) ON DELETE CASCADE
        );
    """)
    cur = conn.execute("PRAGMA table_info(pages)")
    cols = [r[1] for r in cur.fetchall()]
    for col, default in [("seo_og_image", ""), ("seo_og_width", ""), ("seo_og_height", ""),
                          ("seo_twitter_site", ""), ("seo_twitter_card", ""), ("seo_robots", "")]:
        if col not in cols:
            conn.execute(f"ALTER TABLE pages ADD COLUMN {col} TEXT NOT NULL DEFAULT '{default}'")
    if conn.execute("SELECT COUNT(*) FROM settings").fetchone()[0] == 0:
        for k, v in DEFAULT_SETTINGS.items():
            conn.execute("INSERT INTO settings (key, value) VALUES (?, ?)", (k, v))
    conn.commit()
    conn.close()

def get_setting(key, default=""):
    conn = get_conn()
    row = conn.execute("SELECT value FROM settings WHERE key=?", (key,)).fetchone()
    conn.close()
    return row[0] if row else default

def set_setting(key, value):
    conn = get_conn()
    conn.execute("INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)", (key, value))
    conn.commit()
    conn.close()

def get_sections():
    conn = get_conn()
    rows = conn.execute("SELECT id, title, sort_order FROM sections ORDER BY sort_order").fetchall()
    conn.close()
    return rows

def get_pages(section_id=None):
    conn = get_conn()
    if section_id:
        rows = conn.execute("SELECT id, section_id, slug, title, description, keywords, content, sort_order, seo_og_image, seo_og_width, seo_og_height, seo_twitter_site, seo_twitter_card, seo_robots FROM pages WHERE section_id=? ORDER BY sort_order", (section_id,)).fetchall()
    else:
        rows = conn.execute("SELECT id, section_id, slug, title, description, keywords, content, sort_order, seo_og_image, seo_og_width, seo_og_height, seo_twitter_site, seo_twitter_card, seo_robots FROM pages ORDER BY sort_order").fetchall()
    conn.close()
    return rows

def add_section(title):
    conn = get_conn()
    conn.execute("INSERT INTO sections (title, sort_order) VALUES (?, (SELECT COALESCE(MAX(sort_order),0)+1 FROM sections))", (title,))
    conn.commit()
    conn.close()

def add_page(section_id, slug, title):
    conn = get_conn()
    conn.execute("INSERT INTO pages (section_id, slug, title, sort_order) VALUES (?, ?, ?, (SELECT COALESCE(MAX(sort_order),0)+1 FROM pages WHERE section_id=?))", (section_id, slug, title, section_id))
    conn.commit()
    conn.close()

def delete_section(sid):
    conn = get_conn()
    conn.execute("DELETE FROM pages WHERE section_id=?", (sid,))
    conn.execute("DELETE FROM sections WHERE id=?", (sid,))
    conn.commit()
    conn.close()

def delete_page(pid):
    conn = get_conn()
    conn.execute("DELETE FROM pages WHERE id=?", (pid,))
    conn.commit()
    conn.close()

def update_page(pid, slug, title, description, keywords, content, seo_og_image, seo_og_width, seo_og_height, seo_twitter_site, seo_twitter_card, seo_robots):
    conn = get_conn()
    conn.execute("UPDATE pages SET slug=?, title=?, description=?, keywords=?, content=?, seo_og_image=?, seo_og_width=?, seo_og_height=?, seo_twitter_site=?, seo_twitter_card=?, seo_robots=? WHERE id=?",
                 (slug, title, description, keywords, content, seo_og_image, seo_og_width, seo_og_height, seo_twitter_site, seo_twitter_card, seo_robots, pid))
    conn.commit()
    conn.close()

def move_page(pid, direction):
    conn = get_conn()
    row = conn.execute("SELECT id, section_id, sort_order FROM pages WHERE id=?", (pid,)).fetchone()
    if not row: conn.close(); return
    sid, cur_order = row[1], row[2]
    if direction == "up":
        other = conn.execute("SELECT id, sort_order FROM pages WHERE section_id=? AND sort_order<? ORDER BY sort_order DESC LIMIT 1", (sid, cur_order)).fetchone()
    else:
        other = conn.execute("SELECT id, sort_order FROM pages WHERE section_id=? AND sort_order>? ORDER BY sort_order ASC LIMIT 1", (sid, cur_order)).fetchone()
    if not other: conn.close(); return
    conn.execute("UPDATE pages SET sort_order=? WHERE id=?", (other[1], pid))
    conn.execute("UPDATE pages SET sort_order=? WHERE id=?", (cur_order, other[0]))
    conn.commit()
    conn.close()

def move_section(sid, direction):
    conn = get_conn()
    row = conn.execute("SELECT id, sort_order FROM sections WHERE id=?", (sid,)).fetchone()
    if not row: conn.close(); return
    cur_order = row[1]
    if direction == "up":
        other = conn.execute("SELECT id, sort_order FROM sections WHERE sort_order<? ORDER BY sort_order DESC LIMIT 1", (cur_order,)).fetchone()
    else:
        other = conn.execute("SELECT id, sort_order FROM sections WHERE sort_order>? ORDER BY sort_order ASC LIMIT 1", (cur_order,)).fetchone()
    if not other: conn.close(); return
    conn.execute("UPDATE sections SET sort_order=? WHERE id=?", (other[1], sid))
    conn.execute("UPDATE sections SET sort_order=? WHERE id=?", (cur_order, other[0]))
    conn.commit()
    conn.close()

def duplicate_page(pid):
    conn = get_conn()
    row = conn.execute("SELECT section_id, slug, title, description, keywords, content FROM pages WHERE id=?", (pid,)).fetchone()
    if not row: conn.close(); return
    sid, slug, title, desc, kw, content = row
    new_slug = slug + "-copy"
    i = 2
    while conn.execute("SELECT 1 FROM pages WHERE slug=?", (new_slug,)).fetchone():
        new_slug = f"{slug}-copy{i}"; i += 1
    conn.execute("INSERT INTO pages (section_id, slug, title, description, keywords, content, sort_order) VALUES (?, ?, ?, ?, ?, ?, (SELECT COALESCE(MAX(sort_order),0)+1 FROM pages WHERE section_id=?))",
                 (sid, new_slug, title + " (copy)", desc, kw, content, sid))
    conn.commit()
    conn.close()


# ── Build ───────────────────────────────────────────────────────────────────

def esc(text):
    return html.escape(str(text))

def build(log_func=print, progress_func=None):
    total_steps = 5
    def step(n, msg):
        if progress_func:
            progress_func(n / total_steps, msg)
        log_func(msg)

    step(0, "Preparing build...")
    if os.path.exists(BUILD_DIR):
        shutil.rmtree(BUILD_DIR)
    os.makedirs(BUILD_DIR, exist_ok=True)

    site_name = get_setting("SITE_NAME", "Wiki")
    site_url = get_setting("SITE_URL", "").rstrip("/")
    site_desc = get_setting("SITE_DESCRIPTION", "")
    site_author = get_setting("SITE_AUTHOR", "")
    lang = get_setting("DEFAULT_LANG", "en")
    locale = get_setting("DEFAULT_LOCALE", "")
    og_image_global = get_setting("OG_IMAGE", "")
    og_w_global = get_setting("OG_IMAGE_WIDTH", "1200")
    og_h_global = get_setting("OG_IMAGE_HEIGHT", "630")
    twitter_site_global = get_setting("TWITTER_SITE", "")
    twitter_card_global = get_setting("TWITTER_CARD", "summary")
    robots_global = get_setting("ROBOTS", "index, follow")
    favicon_svg_path = get_setting("FAVICON_SVG", "")

    sections = get_sections()
    pages = get_pages()
    today = datetime.date.today().isoformat()

    # Convert SVG favicon to ICO
    ico_generated = False
    if favicon_svg_path and os.path.isfile(favicon_svg_path):
        ico_path = os.path.join(BUILD_DIR, "favicon.ico")
        if svg_to_ico(favicon_svg_path, ico_path):
            ico_generated = True
            log_func("  Converted SVG favicon to favicon.ico")

    def favicon_tag():
        if ico_generated:
            return '<link rel="icon" type="image/x-icon" href="favicon.ico">'
        return '<link rel="icon" href="data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 100 100%22><text y=%22.9em%22 font-size=%2290%22>📖</text></svg>">'

    def sidebar_html(current_slug):
        parts = ['<div class="sidebar-header"><a href="index.html"><h2>' + esc(site_name) + '</h2></a><p>Knowledge Base</p></div>',
                 '<div class="sidebar-search"><input type="text" id="wikiSearch" placeholder="Search pages..."></div>',
                 '<nav><ul>']
        for sid, stitle, _ in sections:
            parts.append('<li><span class="section-title">' + esc(stitle) + '</span><ul>')
            for p in pages:
                if p[1] != sid: continue
                slug, ptitle = p[2], p[3]
                active = ' class="active"' if slug == current_slug else ""
                parts.append('<li><a href="' + esc(slug) + '.html"' + active + '>' + esc(ptitle) + '</a></li>')
            parts.append('</ul></li>')
        parts.append('</ul></nav>')
        parts.append('<div class="search-empty" id="wikiSearchEmpty">No matching pages</div>')
        return "\n".join(parts)

    def json_ld_article(title, desc, url, image, author, published):
        return f'{{"@context":"https://schema.org","@graph":[{{"@type":"Article","headline":"{esc(title)}","description":"{esc(desc)}","url":"{esc(url)}","image":"{esc(image)}","author":{{"@type":"Organization","name":"{esc(author)}"}},"datePublished":"{published}","dateModified":"{published}"}}]}}'

    def json_ld_website():
        return f'{{"@context":"https://schema.org","@type":"WebSite","name":"{esc(site_name)}","url":"{esc(site_url)}/","description":"{esc(site_desc)}","author":{{"@type":"Organization","name":"{esc(site_author)}"}}}}'

    def head(title, desc, keywords, canonical, json_ld, seo_override=None):
        seo_override = seo_override or {}
        og_img = seo_override.get("og_image", "") or og_image_global or f"{site_url}/og-default.png"
        og_w = seo_override.get("og_width", "") or og_w_global or "1200"
        og_h = seo_override.get("og_height", "") or og_h_global or "630"
        tw_site = seo_override.get("twitter_site", "") or twitter_site_global or "@freatevietnam"
        tw_card = seo_override.get("twitter_card", "") or twitter_card_global or "summary_large_image"
        robots_val = seo_override.get("robots", "") or robots_global or "index, follow"
        kw = f'<meta name="keywords" content="{esc(keywords)}">' if keywords else ""
        return f"""<head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0">
<title>{esc(title)} - {esc(site_name)}</title>
<meta name="title" content="{esc(title)} - {esc(site_name)}">
<meta name="description" content="{esc(desc)}">{kw}
<meta name="author" content="{esc(site_author)}">
<meta name="robots" content="{esc(robots_val)}">
<link rel="canonical" href="{esc(canonical)}">
<meta property="og:type" content="article">
<meta property="og:title" content="{esc(title)} - {esc(site_name)}">
<meta property="og:description" content="{esc(desc)}">
<meta property="og:url" content="{esc(canonical)}">
<meta property="og:site_name" content="{esc(site_name)}">
<meta property="og:locale" content="{esc(locale)}">
<meta property="og:image" content="{esc(og_img)}">
<meta property="og:image:width" content="{esc(og_w)}">
<meta property="og:image:height" content="{esc(og_h)}">
<meta name="twitter:card" content="{esc(tw_card)}">
<meta name="twitter:site" content="{esc(tw_site)}">
<meta name="twitter:title" content="{esc(title)} - {esc(site_name)}">
<meta name="twitter:description" content="{esc(desc)}">
<meta name="twitter:image" content="{esc(og_img)}">
<script type="application/ld+json">{json_ld}</script>
{favicon_tag()}
<link rel="stylesheet" href="wiki.css">
<link rel="stylesheet" href="{CDN['highlight_css']}">
<script>MathJax={{ tex: {{ inlineMath: [['$','$']], displayMath: [['$$','$$']] }} }};</script>
<script src="{CDN['mathjax']}" defer></script>
<script src="{CDN['htmx']}"></script></head>"""

    PAGE_CSS = """:root {
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
"""

    PAGE_HTML = """<!DOCTYPE html>
<html lang="LANG">
HEAD
<body hx-boost="true">
<button class="menu-toggle" id="menuToggle">&#9776;</button>
<aside class="sidebar" id="sidebar">SIDEBAR</aside>
<main class="main" id="mainContent">
CONTENT
<footer class="wiki-footer"><p><a href="https://github.com/freatevietnam/freateoj-buildwiki">Edit on GitHub</a> &middot; SITE_NAME</p></footer>
</main>
<script src="{CDN_HLJS}"></script>
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
</script></body></html>"""

    PAGE_HTML = PAGE_HTML.replace("{CDN_HLJS}", CDN["highlight_js"])

    def render_page(p):
        pid, psid, slug, title, desc, keywords, content, _, seo_og_img, seo_og_w, seo_og_h, seo_tw_site, seo_tw_card, seo_robots = p
        canonical = f"{site_url}/{slug}.html"
        content_html = render_md(content)
        ld = json_ld_article(title, desc, canonical, seo_og_img or og_image_global, site_author, today)
        seo_ov = {"og_image": seo_og_img, "og_width": seo_og_w, "og_height": seo_og_h,
                  "twitter_site": seo_tw_site, "twitter_card": seo_tw_card, "robots": seo_robots}
        h = head(title, desc, keywords, canonical, ld, seo_override=seo_ov)
        sb = sidebar_html(slug)
        body = f'<article>{content_html}</article>'
        html_out = PAGE_HTML.replace("LANG", lang).replace("HEAD", h).replace("SIDEBAR", sb).replace("CONTENT", body).replace("SITE_NAME", esc(site_name))
        return html_out

    def render_index():
        cards = ""
        for p in pages:
            pid, psid, slug, ptitle, pdesc = p[0], p[1], p[2], p[3], p[4]
            cards += f'<a href="{esc(slug)}.html" class="wiki-card"><h3>{esc(ptitle)}</h3><p>{esc(pdesc)}</p></a>'
        hero = f'<div class="wiki-hero"><h1>{esc(site_name)}</h1><p class="hero-subtitle">{esc(site_desc)}</p>'
        if pages: hero += f'<p class="hero-count"><span>{len(pages)}</span> articles</p>'
        hero += "</div>"
        grid = f'<div class="wiki-card-grid">{cards}</div>' if cards else ""
        body = hero + grid
        canonical = f"{site_url}/"
        ld = json_ld_website()
        h = head(site_name, site_desc, "", canonical, ld)
        sb = sidebar_html("")
        html_out = PAGE_HTML.replace("LANG", lang).replace("HEAD", h).replace("SIDEBAR", sb).replace("CONTENT", body).replace("SITE_NAME", esc(site_name))
        return html_out

    step(1 / total_steps, f"[1/4] Generating {len(pages)} pages...")
    for p in pages:
        slug = p[2]
        html_out = render_page(p)
        path = os.path.join(BUILD_DIR, slug + ".html")
        with open(path, "w", encoding="utf-8") as f:
            f.write(html_out)
        log_func(f"  -> {slug}.html")

    step(2 / total_steps, "[2/4] Generating index...")
    idx = render_index()
    with open(os.path.join(BUILD_DIR, "index.html"), "w", encoding="utf-8") as f:
        f.write(idx)
    log_func("  -> index.html")

    step(3 / total_steps, "[3/4] Writing CSS...")
    css_path = os.path.join(BUILD_DIR, "wiki.css")
    with open(css_path, "w", encoding="utf-8") as f:
        f.write(PAGE_CSS)
    log_func("  -> wiki.css")

    step(4 / total_steps, "[4/4] Generating sitemap...")
    lines = ['<?xml version="1.0" encoding="UTF-8"?>', '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">']
    sp = ["index.html"] + [p[2] + ".html" for p in pages]
    for p in sp:
        priority = "1.0" if p == "index.html" else "0.8"
        lines.append(f"<url><loc>{esc(site_url)}/{esc(p)}</loc><lastmod>{today}</lastmod><priority>{priority}</priority></url>")
    lines.append("</urlset>")
    with open(os.path.join(BUILD_DIR, "sitemap.xml"), "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")
    log_func("  -> sitemap.xml")

    step(1.0, "Build complete!")


# ── Server ──────────────────────────────────────────────────────────────────

class _ServerState:
    def __init__(self):
        self.httpd = None
        self.thread = None
        self.running = False
        self.port = 0

_server = _ServerState()

def serve_demo(port=8080, log_func=print):
    mimetypes.add_type("text/css", ".css")
    mimetypes.add_type("font/woff2", ".woff2")
    class Handler(http.server.SimpleHTTPRequestHandler):
        def __init__(self, *args, **kwargs):
            super().__init__(*args, directory=BUILD_DIR, **kwargs)
        def log_message(self, fmt, *args):
            parts = " ".join(str(a) for a in args) if args else fmt
            print(f"  [{self.log_date_time_string()}] {parts}")
    _server.httpd = socketserver.TCPServer(("", port), Handler)
    _server.running = True
    _server.port = port
    log_func(f"  Server started: http://localhost:{port}")
    try:
        _server.httpd.serve_forever()
    except OSError:
        pass
    finally:
        _server.running = False
        _server.httpd = None

def stop_server():
    if _server.httpd:
        try:
            _server.httpd.shutdown()
        except OSError:
            pass
        try:
            _server.httpd.server_close()
        except OSError:
            pass
    _server.running = False
    _server.httpd = None

def is_serving():
    return _server.running


# ── GUI ─────────────────────────────────────────────────────────────────────

BG      = "#fafbfe"
BG2     = "#f0f2f8"
SURFACE = "#ffffff"
SURFACE2 = "#f4f5fa"
SURFACE3 = "#eceef5"
TEXT    = "#1a1d2e"
TEXT2   = "#4a4e6a"
TEXT3   = "#8085a0"
ACCENT  = "#6c5ce7"
ACCENT2 = "#7c6cf7"
ACCENT3 = "#a29bfe"
GOLD    = "#e17055"
GOLD2   = "#d63031"
BORDER  = "#e2e5f0"
BORDER2 = "#d0d4e0"
SUCCESS = "#00b894"
SUCCESS2 = "#00a381"
DANGER  = "#d63031"
DANGER2 = "#b71c1c"
WARNING = "#fdcb6e"
ACCENT_HOVER = "#5a4bd1"

class WikiApp:
    def __init__(self):
        if not _GUI_OK:
            raise ImportError("Tkinter not available")

        ctk.set_appearance_mode("light")
        ctk.set_default_color_theme("blue")
        self.root = ctk.CTk()
        self.root.title("FreateOJ Wiki Builder")
        self.root.geometry("1200x750")
        self.root.minsize(900, 600)
        self.current_page_id = None
        self.mbox = messagebox
        self.sdialog = _sd
        self.build_start_time = 0

        self.menu = tk.Menu(self.root)
        self.root.config(menu=self.menu)
        file_menu = tk.Menu(self.menu, tearoff=0)
        file_menu.add_command(label="Build", command=self.run_build, accelerator="Ctrl+B")
        file_menu.add_separator()
        file_menu.add_command(label="Export DB", command=self.export_db)
        file_menu.add_command(label="Import DB", command=self.import_db)
        file_menu.add_separator()
        file_menu.add_command(label="Exit", command=self.root.quit)
        self.menu.add_cascade(label="File", menu=file_menu)
        self.root.bind("<Control-b>", lambda e: self.run_build())

        self.root.grid_columnconfigure(0, weight=0)
        self.root.grid_columnconfigure(1, weight=1)
        self.root.grid_rowconfigure(0, weight=1)

        # ── Left panel ──
        self.left = ctk.CTkFrame(self.root, width=260, corner_radius=0, fg_color=SURFACE)
        self.left.grid(row=0, column=0, sticky="nswe")
        self.left.grid_rowconfigure(1, weight=1)
        self.left.grid_columnconfigure(0, weight=1)

        header = ctk.CTkFrame(self.left, fg_color="transparent")
        header.grid(row=0, column=0, padx=16, pady=(16,8), sticky="ew")
        header.grid_columnconfigure(0, weight=1)
        ctk.CTkLabel(header, text="📖 FreateOJ Wiki", font=ctk.CTkFont(size=16, weight="bold"), text_color=ACCENT).grid(row=0, column=0, sticky="w")
        self.page_count_label = ctk.CTkLabel(header, text="0 pages", font=ctk.CTkFont(size=11), text_color=TEXT2)
        self.page_count_label.grid(row=1, column=0, sticky="w")

        tree_frame = ctk.CTkFrame(self.left, fg_color="transparent")
        tree_frame.grid(row=1, column=0, padx=8, pady=4, sticky="nswe")
        tree_frame.grid_rowconfigure(0, weight=1)
        tree_frame.grid_columnconfigure(0, weight=1)

        self.tree = ttk.Treeview(tree_frame, columns=(), show="tree", selectmode="browse")
        style = ttk.Style()
        style.theme_use("clam")
        style.configure("Treeview", background=SURFACE2, foreground=TEXT, fieldbackground=SURFACE2, borderwidth=0, font=("Segoe UI", 12))
        style.configure("Treeview.Heading", font=("Segoe UI", 12, "bold"))
        style.map("Treeview", background=[("selected", ACCENT)], foreground=[("selected", "#fff")])
        self.tree.grid(row=0, column=0, sticky="nswe")
        self.tree.bind("<<TreeviewSelect>>", self.on_tree_select)

        tree_btn = ctk.CTkFrame(self.left, fg_color="transparent")
        tree_btn.grid(row=2, column=0, padx=8, pady=(4,12), sticky="ew")
        tree_btn.grid_columnconfigure((0,1,2,3,4), weight=1)
        ctk.CTkButton(tree_btn, text="📁", width=36, command=self.add_section_dlg, fg_color=SURFACE2, hover_color=SURFACE3, text_color=ACCENT, font=ctk.CTkFont(size=16)).grid(row=0, column=0, padx=2)
        ctk.CTkButton(tree_btn, text="📄", width=36, command=self.add_page_dlg, fg_color=SURFACE2, hover_color=SURFACE3, text_color=ACCENT, font=ctk.CTkFont(size=16)).grid(row=0, column=1, padx=2)
        ctk.CTkButton(tree_btn, text="📋", width=36, command=self.duplicate_page_dlg, fg_color=SURFACE2, hover_color=SURFACE3, text_color=TEXT2, font=ctk.CTkFont(size=16)).grid(row=0, column=2, padx=2)
        ctk.CTkButton(tree_btn, text="↑", width=36, command=lambda: self.move_sel("up"), fg_color=SURFACE2, hover_color=SURFACE3, text_color=TEXT2, font=ctk.CTkFont(size=16)).grid(row=0, column=3, padx=2)
        ctk.CTkButton(tree_btn, text="↓", width=36, command=lambda: self.move_sel("down"), fg_color=SURFACE2, hover_color=SURFACE3, text_color=TEXT2, font=ctk.CTkFont(size=16)).grid(row=0, column=4, padx=2)

        ctk.CTkButton(self.left, text="🗑 Delete", height=32, fg_color=DANGER, hover_color=DANGER2, command=self.delete_item).grid(row=3, column=0, padx=8, pady=(0,12), sticky="ew")

        # ── Right panel ──
        self.right = ctk.CTkTabview(self.root, fg_color=BG)
        self.right.grid(row=0, column=1, padx=8, pady=8, sticky="nswe")

        self.tab_edit = self.right.add("✏ Editor")
        self.tab_preview = self.right.add("👁 Preview")
        self.tab_seo = self.right.add("⚙ Page SEO")
        self.tab_seo_global = self.right.add("🌐 Global SEO")
        self.tab_stats = self.right.add("📊 Stats")
        self.tab_build = self.right.add("🔨 Build")

        self._build_edit_tab()
        self._build_preview_tab()
        self._build_page_seo_tab()
        self._build_global_seo_tab()
        self._build_stats_tab()
        self._build_build_tab()

        # ── Status bar ──
        status_frame = ctk.CTkFrame(self.root, fg_color=SURFACE, corner_radius=0, height=32)
        status_frame.grid(row=1, column=0, columnspan=2, sticky="ew")
        status_frame.grid_columnconfigure(1, weight=1)
        self.status = ctk.CTkLabel(status_frame, text="Ready", font=ctk.CTkFont(size=11), text_color=TEXT2, anchor="w")
        self.status.grid(row=0, column=0, padx=12, pady=4, sticky="w")
        self.serve_btn = ctk.CTkButton(status_frame, text="▶ Start Serve", width=120, height=26,
                                        fg_color=SUCCESS, hover_color=SUCCESS2,
                                        font=ctk.CTkFont(size=11), command=self.toggle_serve)
        self.serve_btn.grid(row=0, column=1, padx=12, pady=4, sticky="e")

        self._refresh_tree()
        init_db()

    def _build_edit_tab(self):
        f = self.tab_edit
        for i in range(6): f.grid_rowconfigure(i, weight=0)
        f.grid_rowconfigure(4, weight=1)
        f.grid_columnconfigure(1, weight=1)

        label_font = ctk.CTkFont(size=12, weight="bold")
        entry_fg = SURFACE2

        ctk.CTkLabel(f, text="Slug", font=label_font, text_color=ACCENT).grid(row=0, column=0, padx=12, pady=(12,4), sticky="w")
        self.entry_slug = ctk.CTkEntry(f, placeholder_text="url-friendly-name", fg_color=entry_fg, border_color=BORDER)
        self.entry_slug.grid(row=0, column=1, padx=8, pady=(12,4), sticky="ew")

        ctk.CTkLabel(f, text="Title", font=label_font, text_color=ACCENT).grid(row=1, column=0, padx=12, pady=4, sticky="w")
        self.entry_title = ctk.CTkEntry(f, placeholder_text="Page title", fg_color=entry_fg, border_color=BORDER)
        self.entry_title.grid(row=1, column=1, padx=8, pady=4, sticky="ew")

        ctk.CTkLabel(f, text="Description", font=label_font, text_color=ACCENT).grid(row=2, column=0, padx=12, pady=4, sticky="w")
        self.entry_desc = ctk.CTkEntry(f, placeholder_text="Short description for SEO", fg_color=entry_fg, border_color=BORDER)
        self.entry_desc.grid(row=2, column=1, padx=8, pady=4, sticky="ew")

        ctk.CTkLabel(f, text="Keywords", font=label_font, text_color=ACCENT).grid(row=3, column=0, padx=12, pady=4, sticky="w")
        self.entry_kw = ctk.CTkEntry(f, placeholder_text="keyword1, keyword2, ...", fg_color=entry_fg, border_color=BORDER)
        self.entry_kw.grid(row=3, column=1, padx=8, pady=4, sticky="ew")

        ctk.CTkLabel(f, text="Content (Markdown)", font=label_font, text_color=ACCENT).grid(row=4, column=0, padx=12, pady=(8,0), sticky="nw")
        self.text_content = ctk.CTkTextbox(f, wrap="word", font=ctk.CTkFont(family="Consolas,monospace", size=13), fg_color=entry_fg, border_color=BORDER)
        self.text_content.grid(row=4, column=1, padx=8, pady=(8,8), sticky="nswe")

        save_frame = ctk.CTkFrame(f, fg_color="transparent")
        save_frame.grid(row=5, column=1, padx=8, pady=(0,8), sticky="e")
        ctk.CTkButton(save_frame, text="💾 Save Page", command=self.save_page, fg_color=ACCENT, hover_color=ACCENT_HOVER).pack(side="right")
        ctk.CTkButton(save_frame, text="👁 Preview", command=lambda: self.right.set("👁 Preview"), fg_color=SURFACE2, hover_color=SURFACE3, text_color=TEXT2).pack(side="right", padx=(0,8))

    def _build_preview_tab(self):
        f = self.tab_preview
        f.grid_rowconfigure(0, weight=1)
        f.grid_columnconfigure(0, weight=1)
        self.preview_text = ctk.CTkTextbox(f, wrap="word", font=ctk.CTkFont(family="Consolas,monospace", size=13), fg_color=SURFACE2, border_color=BORDER, state="disabled")
        self.preview_text.grid(row=0, column=0, padx=8, pady=8, sticky="nswe")

    def _build_page_seo_tab(self):
        f = self.tab_seo
        f.grid_columnconfigure(1, weight=1)
        self.page_seo_entries = {}
        keys = [
            ("seo_og_image", "OG Image URL (leave empty = global)"),
            ("seo_og_width", "OG Image Width"),
            ("seo_og_height", "OG Image Height"),
            ("seo_twitter_site", "Twitter Site"),
            ("seo_twitter_card", "Twitter Card Type"),
            ("seo_robots", "Robots"),
        ]
        label_font = ctk.CTkFont(size=12, weight="bold")
        for i, (key, label) in enumerate(keys):
            ctk.CTkLabel(f, text=label, font=label_font, text_color=ACCENT, anchor="w").grid(row=i, column=0, padx=12, pady=6, sticky="w")
            e = ctk.CTkEntry(f, fg_color=SURFACE2, border_color=BORDER, placeholder_text="empty = use global setting")
            e.grid(row=i, column=1, padx=8, pady=6, sticky="ew")
            self.page_seo_entries[key] = e
        ctk.CTkButton(f, text="💾 Save Page SEO", command=self.save_page_seo, fg_color=ACCENT, hover_color=ACCENT_HOVER).grid(row=len(keys), column=1, padx=8, pady=16, sticky="e")

    def _build_global_seo_tab(self):
        f = self.tab_seo_global
        f.grid_columnconfigure(1, weight=1)
        self.seo_entries = {}
        keys = ["SITE_NAME", "SITE_URL", "SITE_DESCRIPTION", "SITE_AUTHOR", "DEFAULT_LANG", "DEFAULT_LOCALE",
                "OG_IMAGE", "OG_IMAGE_WIDTH", "OG_IMAGE_HEIGHT", "TWITTER_SITE", "TWITTER_CARD", "ROBOTS"]
        label_font = ctk.CTkFont(size=12, weight="bold")
        for i, k in enumerate(keys):
            ctk.CTkLabel(f, text=k, font=label_font, text_color=ACCENT, anchor="w").grid(row=i, column=0, padx=12, pady=6, sticky="w")
            e = ctk.CTkEntry(f, fg_color=SURFACE2, border_color=BORDER)
            e.grid(row=i, column=1, padx=8, pady=6, sticky="ew")
            e.insert(0, get_setting(k))
            self.seo_entries[k] = e

        row = len(keys)
        ctk.CTkLabel(f, text="FAVICON_SVG", font=label_font, text_color=ACCENT, anchor="w").grid(row=row, column=0, padx=12, pady=6, sticky="w")
        fav_frame = ctk.CTkFrame(f, fg_color="transparent")
        fav_frame.grid(row=row, column=1, padx=8, pady=6, sticky="ew")
        fav_frame.grid_columnconfigure(0, weight=1)
        self.entry_favicon = ctk.CTkEntry(fav_frame, fg_color=SURFACE2, border_color=BORDER, placeholder_text="path/to/logo.svg")
        self.entry_favicon.grid(row=0, column=0, sticky="ew")
        self.entry_favicon.insert(0, get_setting("FAVICON_SVG"))
        ctk.CTkButton(fav_frame, text="Browse", width=70, command=self._browse_favicon, fg_color=SURFACE3, hover_color=BORDER, text_color=TEXT2).grid(row=0, column=1, padx=(6,0))

        self.favicon_status = ctk.CTkLabel(f, text="", font=ctk.CTkFont(size=11), text_color=TEXT2, anchor="w")
        self.favicon_status.grid(row=row+1, column=1, padx=8, sticky="w")
        if _SVG_OK:
            self.favicon_status.configure(text="CairoSVG + Pillow available", text_color=SUCCESS)
        else:
            self.favicon_status.configure(text="Install: pip install cairosvg Pillow", text_color=DANGER)

        ctk.CTkButton(f, text="💾 Save Global SEO", command=self.save_seo, fg_color=ACCENT, hover_color=ACCENT_HOVER).grid(row=row+2, column=1, padx=8, pady=16, sticky="e")

    def _browse_favicon(self):
        path = filedialog.askopenfilename(filetypes=[("SVG files", "*.svg"), ("All files", "*.*")], title="Select SVG Logo")
        if path:
            self.entry_favicon.delete(0, "end")
            self.entry_favicon.insert(0, path)

    def _build_stats_tab(self):
        f = self.tab_stats
        f.grid_columnconfigure(0, weight=1)
        f.grid_rowconfigure(1, weight=1)

        header = ctk.CTkLabel(f, text="Page Statistics", font=ctk.CTkFont(size=16, weight="bold"), text_color=ACCENT)
        header.grid(row=0, column=0, padx=16, pady=(16,8), sticky="w")

        stats_frame = ctk.CTkFrame(f, fg_color=SURFACE, corner_radius=10)
        stats_frame.grid(row=1, column=0, padx=12, pady=8, sticky="nswe")
        stats_frame.grid_columnconfigure((0,1), weight=1)

        self.stats_labels = {}
        stat_items = [
            ("total_pages", "Total Pages"),
            ("total_sections", "Total Sections"),
            ("total_words", "Total Words"),
            ("total_chars", "Total Characters"),
            ("avg_words", "Avg Words/Page"),
            ("reading_time", "Est. Reading Time"),
            ("build_size", "Build Size"),
            ("last_built", "Last Built"),
        ]
        for i, (key, label) in enumerate(stat_items):
            row, col = divmod(i, 2)
            card = ctk.CTkFrame(stats_frame, fg_color=SURFACE2, corner_radius=8)
            card.grid(row=row, column=col, padx=8, pady=6, sticky="nswe")
            ctk.CTkLabel(card, text=label, font=ctk.CTkFont(size=11), text_color=TEXT2).pack(padx=12, pady=(8,2), anchor="w")
            lbl = ctk.CTkLabel(card, text="—", font=ctk.CTkFont(size=18, weight="bold"), text_color=TEXT)
            lbl.pack(padx=12, pady=(0,8), anchor="w")
            self.stats_labels[key] = lbl

        ctk.CTkButton(f, text="🔄 Refresh Stats", command=self._refresh_stats, fg_color=ACCENT, hover_color=ACCENT_HOVER).grid(row=2, column=0, padx=12, pady=8)
        self._refresh_stats()

    def _refresh_stats(self):
        sections = get_sections()
        pages = get_pages()
        total_words = 0
        total_chars = 0
        for p in pages:
            content = p[6] or ""
            total_chars += len(content)
            total_words += len(content.split())
        avg = total_words // len(pages) if pages else 0
        reading = max(1, total_words // 200)

        build_size = "—"
        if os.path.exists(BUILD_DIR):
            total = 0
            for dirpath, _, filenames in os.walk(BUILD_DIR):
                for fn in filenames:
                    total += os.path.getsize(os.path.join(dirpath, fn))
            if total > 1024 * 1024:
                build_size = f"{total / 1024 / 1024:.1f} MB"
            elif total > 1024:
                build_size = f"{total / 1024:.1f} KB"
            else:
                build_size = f"{total} B"

        last_built = "—"
        idx_path = os.path.join(BUILD_DIR, "index.html")
        if os.path.exists(idx_path):
            ts = os.path.getmtime(idx_path)
            last_built = datetime.datetime.fromtimestamp(ts).strftime("%Y-%m-%d %H:%M")

        self.stats_labels["total_pages"].configure(text=str(len(pages)))
        self.stats_labels["total_sections"].configure(text=str(len(sections)))
        self.stats_labels["total_words"].configure(text=f"{total_words:,}")
        self.stats_labels["total_chars"].configure(text=f"{total_chars:,}")
        self.stats_labels["avg_words"].configure(text=str(avg))
        self.stats_labels["reading_time"].configure(text=f"{reading} min")
        self.stats_labels["build_size"].configure(text=build_size)
        self.stats_labels["last_built"].configure(text=last_built)

    def _build_build_tab(self):
        f = self.tab_build
        f.grid_rowconfigure(2, weight=1)
        f.grid_columnconfigure(0, weight=1)

        btn_frame = ctk.CTkFrame(f, fg_color="transparent")
        btn_frame.grid(row=0, column=0, padx=8, pady=8, sticky="ew")
        ctk.CTkButton(btn_frame, text="🔨 Build", command=self.run_build, fg_color=ACCENT, hover_color=ACCENT_HOVER).pack(side="left", padx=4)
        ctk.CTkButton(btn_frame, text="📂 Open build/", command=lambda: os.startfile(BUILD_DIR) if sys.platform == "win32" else os.system(f"xdg-open {BUILD_DIR} 2>/dev/null &"), fg_color=SURFACE2, hover_color=SURFACE3, text_color=TEXT2).pack(side="left", padx=4)

        progress_frame = ctk.CTkFrame(f, fg_color="transparent")
        progress_frame.grid(row=1, column=0, padx=8, pady=(0,4), sticky="ew")
        progress_frame.grid_columnconfigure(0, weight=1)
        self.progress_bar = ctk.CTkProgressBar(progress_frame, height=6, fg_color=SURFACE2, progress_color=ACCENT)
        self.progress_bar.grid(row=0, column=0, sticky="ew", padx=4)
        self.progress_bar.set(0)
        self.progress_label = ctk.CTkLabel(progress_frame, text="", font=ctk.CTkFont(size=11), text_color=TEXT2)
        self.progress_label.grid(row=1, column=0, sticky="w", padx=8)

        self.log_text = ctk.CTkTextbox(f, wrap="word", font=ctk.CTkFont(family="Consolas,monospace", size=12), fg_color=SURFACE2, border_color=BORDER)
        self.log_text.grid(row=2, column=0, padx=8, pady=(0,8), sticky="nswe")

    def log(self, msg):
        ts = datetime.datetime.now().strftime("%H:%M:%S")
        self.log_text.insert("end", f"[{ts}] {msg}\n")
        self.log_text.see("end")
        self.status.configure(text=msg)

    def _progress(self, value, msg):
        try:
            self.progress_bar.set(value)
            self.progress_label.configure(text=msg)
        except Exception:
            pass

    def _refresh_tree(self):
        for item in self.tree.get_children():
            self.tree.delete(item)
        sections = get_sections()
        pages = get_pages()
        total = 0
        for sid, stitle, _ in sections:
            sid_str = f"s:{sid}"
            self.tree.insert("", "end", iid=sid_str, text=f"📁 {stitle}", tags=("section",))
            count = 0
            for p in pages:
                if p[1] != sid: continue
                pid, ptitle = p[0], p[3]
                self.tree.insert(sid_str, "end", iid=f"p:{pid}", text=f"  {ptitle}", tags=("page",))
                count += 1
            total += count
            self.tree.item(sid_str, text=f"📁 {stitle} ({count})")
        self.page_count_label.configure(text=f"{total} pages in {len(sections)} sections")

    def on_tree_select(self, evt):
        sel = self.tree.selection()
        if not sel: return
        iid = sel[0]
        if iid.startswith("p:"):
            pid = int(iid[2:])
            self.current_page_id = pid
            conn = get_conn()
            row = conn.execute("SELECT slug, title, description, keywords, content, seo_og_image, seo_og_width, seo_og_height, seo_twitter_site, seo_twitter_card, seo_robots FROM pages WHERE id=?", (pid,)).fetchone()
            conn.close()
            if row:
                self.entry_slug.delete(0, "end"); self.entry_slug.insert(0, row[0])
                self.entry_title.delete(0, "end"); self.entry_title.insert(0, row[1])
                self.entry_desc.delete(0, "end"); self.entry_desc.insert(0, row[2])
                self.entry_kw.delete(0, "end"); self.entry_kw.insert(0, row[3])
                self.text_content.delete("0.0", "end"); self.text_content.insert("0.0", row[4])
                self._update_preview(row[4])
                seo_keys = ["seo_og_image", "seo_og_width", "seo_og_height", "seo_twitter_site", "seo_twitter_card", "seo_robots"]
                for i, k in enumerate(seo_keys):
                    self.page_seo_entries[k].delete(0, "end")
                    self.page_seo_entries[k].insert(0, row[5+i])
                self.status.configure(text=f"Editing: {row[1]}")
                self.right.set("✏ Editor")

    def _update_preview(self, md_text):
        h = render_md(md_text)
        self.preview_text.configure(state="normal")
        self.preview_text.delete("0.0", "end")
        self.preview_text.insert("0.0", h)
        self.preview_text.configure(state="disabled")

    def save_page(self):
        if not self.current_page_id:
            self.mbox.showwarning("No page selected", "Select a page from the sidebar first.")
            return
        slug = self.entry_slug.get().strip()
        title = self.entry_title.get().strip()
        desc = self.entry_desc.get().strip()
        kw = self.entry_kw.get().strip()
        content = self.text_content.get("0.0", "end").strip()
        if not slug or not title:
            self.mbox.showwarning("Required", "Slug and title are required.")
            return
        conn = get_conn()
        row = conn.execute("SELECT seo_og_image, seo_og_width, seo_og_height, seo_twitter_site, seo_twitter_card, seo_robots FROM pages WHERE id=?", (self.current_page_id,)).fetchone()
        conn.close()
        seo_vals = [row[i] if row and row[i] else "" for i in range(6)] if row else [""]*6
        update_page(self.current_page_id, slug, title, desc, kw, content, *seo_vals)
        self._refresh_tree()
        self.log(f"Saved: {title}")
        self.status.configure(text=f"Saved: {title}")

    def save_page_seo(self):
        if not self.current_page_id:
            self.mbox.showwarning("No page selected", "Select a page first.")
            return
        vals = [self.page_seo_entries[k].get().strip() for k in ["seo_og_image", "seo_og_width", "seo_og_height", "seo_twitter_site", "seo_twitter_card", "seo_robots"]]
        conn = get_conn()
        row = conn.execute("SELECT slug, title, description, keywords, content FROM pages WHERE id=?", (self.current_page_id,)).fetchone()
        conn.close()
        if row:
            update_page(self.current_page_id, row[0], row[1], row[2], row[3], row[4], *vals)
            self.log("Page SEO saved.")

    def save_seo(self):
        for k, e in self.seo_entries.items():
            set_setting(k, e.get().strip())
        set_setting("FAVICON_SVG", self.entry_favicon.get().strip())
        self.log("Global SEO + favicon saved.")

    def add_section_dlg(self):
        name = self.sdialog.askstring("New Section", "Section title:", parent=self.root)
        if name and name.strip():
            add_section(name.strip())
            self._refresh_tree()
            self.log(f"Added section: {name.strip()}")

    def add_page_dlg(self):
        sections = get_sections()
        if not sections:
            self.mbox.showwarning("No sections", "Add a section first.")
            return
        names = [s[1] for s in sections]
        sec = self.sdialog.askstring("New Page", "Section name:\n" + ", ".join(names), parent=self.root)
        if not sec: return
        sid = None
        for s in sections:
            if s[1].lower() == sec.strip().lower():
                sid = s[0]; break
        if sid is None:
            self.mbox.showerror("Not found", f"No section named '{sec}'")
            return
        title = self.sdialog.askstring("New Page", "Page title:", parent=self.root)
        if not title: return
        slug = self.sdialog.askstring("New Page", "Slug (URL name):", parent=self.root)
        if not slug: return
        add_page(sid, slug.strip(), title.strip())
        self._refresh_tree()
        self.log(f"Added page: {title.strip()}")

    def duplicate_page_dlg(self):
        if not self.current_page_id:
            self.mbox.showwarning("No page", "Select a page to duplicate.")
            return
        duplicate_page(self.current_page_id)
        self._refresh_tree()
        self.log("Page duplicated.")

    def move_sel(self, direction):
        sel = self.tree.selection()
        if not sel: return
        iid = sel[0]
        if iid.startswith("p:"):
            move_page(int(iid[2:]), direction)
            self._refresh_tree()
        elif iid.startswith("s:"):
            move_section(int(iid[2:]), direction)
            self._refresh_tree()

    def delete_item(self):
        sel = self.tree.selection()
        if not sel: return
        iid = sel[0]
        if iid.startswith("s:"):
            sid = int(iid[2:])
            if self.mbox.askyesno("Delete section", "Delete this section and all its pages?"):
                delete_section(sid)
                self.current_page_id = None
                self._refresh_tree()
                self.log("Deleted section")
        elif iid.startswith("p:"):
            pid = int(iid[2:])
            if self.mbox.askyesno("Delete page", "Delete this page?"):
                delete_page(pid)
                self.current_page_id = None
                self._refresh_tree()
                self.log("Deleted page")

    def export_db(self):
        path = filedialog.asksaveasfilename(defaultextension=".db", filetypes=[("SQLite DB", "*.db")], title="Export DB")
        if path:
            shutil.copy2(DB_PATH, path)
            self.log(f"Exported DB to {path}")

    def import_db(self):
        path = filedialog.askopenfilename(filetypes=[("SQLite DB", "*.db")], title="Import DB")
        if path:
            shutil.copy2(path, DB_PATH)
            self._refresh_tree()
            self.log(f"Imported DB from {path}")

    def toggle_serve(self):
        if is_serving():
            stop_server()
            self.serve_btn.configure(text="▶ Start Serve", fg_color=SUCCESS, hover_color=SUCCESS2)
            self.status.configure(text="Server stopped.")
            self.log("Server stopped.")
        else:
            def task():
                build(log_func=self.log, progress_func=self._progress)
                serve_demo(port=8080, log_func=self.log)
            threading.Thread(target=task, daemon=True).start()
            self.serve_btn.configure(text="■ Stop Serve", fg_color=DANGER, hover_color=DANGER2)
            self.status.configure(text="Serving on :8080")

    def run_build(self):
        def task():
            self.build_start_time = time.time()
            self.log("--- Build started ---")
            self.status.configure(text="Building...")
            self.progress_bar.set(0)
            try:
                build(log_func=self.log, progress_func=self._progress)
                elapsed = time.time() - self.build_start_time
                self.status.configure(text=f"Build complete! ({elapsed:.1f}s)")
                self._refresh_stats()
            except Exception as e:
                self.log(f"ERROR: {e}")
                self.status.configure(text="Build failed!")
        threading.Thread(target=task, daemon=True).start()

    def run(self):
        self.root.mainloop()


if __name__ == "__main__":
    init_db()
    if "--cli" in sys.argv:
        build()
        if "--serve" in sys.argv:
            port = 8080
            for i, arg in enumerate(sys.argv):
                if arg == "--port" and i + 1 < len(sys.argv):
                    try: port = int(sys.argv[i + 1])
                    except: pass
            serve_demo(port=port)
    else:
        if not _GUI_OK:
            print("GUI unavailable (Tkinter not found). Use --cli to build.", file=sys.stderr)
            print("Install Tk: sudo apt install python3-tk", file=sys.stderr)
            sys.exit(1)
        app = WikiApp()
        app.run()
