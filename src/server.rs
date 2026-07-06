use anyhow::Result;
use std::fs;
use std::io::Read;
use std::path::Path;
use tiny_http::{Server, Response, Header};

use crate::build;
use crate::db;

fn mime(path: &str) -> &str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "txt" => "text/plain",
        _ => "application/octet-stream",
    }
}

fn ok(data: &str, ct: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let h = Header::from_bytes(b"Content-Type", format!("{}; charset=utf-8", ct).as_bytes()).unwrap();
    Response::from_string(data).with_header(h)
}

fn ok_json(data: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    ok(data, "application/json")
}

fn not_found() -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string("404").with_status_code(404)
}

fn redirect(url: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let h = Header::from_bytes(b"Location", url.as_bytes()).unwrap();
    Response::from_string("").with_status_code(302).with_header(h)
}

// ─── Admin HTML Template ─────────────────────────────────────────────────────

fn admin_html() -> &'static str {
    r#"<!DOCTYPE html>
<html lang="vi">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1.0">
<title>FreateOJ Wiki Admin</title>
<style>
:root{--bg:#0d0d1a;--bg2:#151528;--bg3:#1c1c38;--surface:#1a1a30;--hover:#28284f;--active:#33336b;--text:#d0d0e0;--text2:#9090b0;--text3:#585880;--accent:#7c7cf0;--accent2:#9999ff;--success:#22c55e;--danger:#ef4444;--border:#2a2a4a;--radius:8px}
*{margin:0;padding:0;box-sizing:border-box}
body{font-family:'Inter',system-ui,-apple-system,sans-serif;background:var(--bg);color:var(--text);display:flex;min-height:100vh;line-height:1.6;font-size:14px}
/* Sidebar */
.sidebar{width:260px;background:var(--bg2);border-right:1px solid var(--border);height:100vh;position:sticky;top:0;overflow-y:auto;flex-shrink:0}
.sidebar-header{padding:20px 16px;border-bottom:1px solid var(--border)}
.sidebar-header h2{color:var(--accent);font-size:15px;font-weight:700}
.sidebar-header p{color:var(--text3);font-size:11px;margin-top:2px}
.sidebar-nav{padding:8px 0}
.nav-item,.nav-sub{display:block;width:100%;padding:8px 16px;background:none;border:none;color:var(--text2);font-size:13px;cursor:pointer;text-align:left;transition:all .15s}
.nav-item:hover,.nav-sub:hover{background:var(--hover);color:var(--text)}
.nav-item.active{color:var(--accent);background:var(--active)}
.nav-section{padding:12px 16px 4px;color:var(--text3);font-size:10px;text-transform:uppercase;letter-spacing:1.5px;font-weight:700}
.nav-sub{padding-left:28px;font-size:12px}
.nav-sub.active{color:var(--accent);background:var(--active)}
/* Main */
.main{flex:1;padding:24px;max-width:1200px}
h1{font-size:22px;margin-bottom:8px;font-weight:700}
h2{font-size:16px;margin:20px 0 12px;font-weight:600;color:var(--accent)}
.card{background:var(--surface);border:1px solid var(--border);border-radius:var(--radius);padding:20px;margin-bottom:16px}
.card h3{font-size:13px;color:var(--text3);margin-bottom:4px}
.card .val{font-size:20px;font-weight:700}
.row{display:flex;gap:12px;flex-wrap:wrap}
.stat{flex:1;min-width:140px}
/* Form */
.form-group{margin-bottom:12px}
label{display:block;color:var(--text2);font-size:12px;margin-bottom:4px;font-weight:600}
input,textarea,select{width:100%;padding:10px 12px;background:var(--bg);border:1px solid var(--border);border-radius:var(--radius);color:var(--text);font-size:13px;outline:none;transition:border-color .2s}
input:focus,textarea:focus{border-color:var(--accent)}
textarea{font-family:'JetBrains Mono','Fira Code',monospace;font-size:12px;min-height:300px;resize:vertical}
.btn{display:inline-block;padding:10px 18px;border:none;border-radius:var(--radius);font-size:13px;font-weight:600;cursor:pointer;transition:all .15s;text-decoration:none;line-height:1}
.btn-primary{background:var(--accent);color:#fff}
.btn-primary:hover{background:var(--accent2)}
.btn-success{background:var(--success);color:#fff}
.btn-success:hover{opacity:.85}
.btn-danger{background:var(--danger);color:#fff}
.btn-danger:hover{opacity:.85}
.btn-ghost{background:transparent;color:var(--text2);padding:6px 10px;font-size:12px}
.btn-ghost:hover{background:var(--hover);color:var(--text)}
.btn-sm{padding:6px 12px;font-size:12px}
.btn-group{display:flex;gap:6px;flex-wrap:wrap;margin-top:12px}
/* Table */
table{width:100%;border-collapse:collapse}
th,td{padding:10px 12px;text-align:left;border-bottom:1px solid var(--border)}
th{color:var(--text3);font-size:11px;text-transform:uppercase;letter-spacing:1px;font-weight:700}
td{font-size:13px;color:var(--text2)}
tr:hover td{background:var(--hover)}
/* Toast */
.toast{position:fixed;top:20px;right:20px;z-index:9999}
.toast-inner{background:var(--surface);border:1px solid var(--border);border-radius:var(--radius);padding:14px 20px;margin-bottom:8px;box-shadow:0 4px 20px rgba(0,0,0,.4);transform:translateX(120%);transition:transform .3s ease;max-width:360px}
.toast-inner.show{transform:translateX(0)}
.toast-inner.success{border-left:3px solid var(--success)}
.toast-inner.error{border-left:3px solid var(--danger)}
.toast-inner.info{border-left:3px solid var(--accent)}
.toast-inner .msg{color:var(--text);font-size:13px}
/* Loading */
.spinner{display:inline-block;width:16px;height:16px;border:2px solid var(--border);border-top-color:var(--accent);border-radius:50%;animation:spin .6s linear infinite;vertical-align:middle;margin-right:6px}
@keyframes spin{to{transform:rotate(360deg)}}
/* Page list */
.page-row{display:flex;align-items:center;justify-content:space-between;padding:8px 12px;border-radius:var(--radius);transition:background .15s;margin:2px 0}
.page-row:hover{background:var(--hover)}
.page-row .title{color:var(--text);font-weight:500}
.page-row .actions{display:flex;gap:4px;opacity:0;transition:opacity .15s}
.page-row:hover .actions{opacity:1}
.section-title{padding:16px 12px 6px;color:var(--text3);font-size:11px;text-transform:uppercase;letter-spacing:1px;font-weight:700}
.empty{color:var(--text3);padding:20px;text-align:center}
/* Badge */
.badge{display:inline-block;padding:2px 8px;border-radius:12px;font-size:11px;font-weight:600}
.badge-accent{background:var(--active);color:var(--accent)}
.badge-success{background:rgba(34,197,94,.15);color:var(--success)}
/* Tabs in admin */
.tabs{display:flex;gap:2px;margin-bottom:16px;border-bottom:1px solid var(--border)}
.tab{padding:10px 18px;border:none;background:none;color:var(--text2);cursor:pointer;font-size:13px;border-bottom:2px solid transparent;transition:all .15s}
.tab:hover{color:var(--text)}
.tab.active{color:var(--accent);border-bottom-color:var(--accent)}
.tab-content{display:none}
.tab-content.active{display:block}
/* Progress bar */
.progress{height:6px;background:var(--bg3);border-radius:3px;overflow:hidden;margin:8px 0}
.progress-inner{height:100%;background:var(--accent);border-radius:3px;transition:width .3s ease;width:0%}
.progress-text{font-size:12px;color:var(--text3);margin-top:4px}
/* Log */
.log-box{background:var(--bg);border:1px solid var(--border);border-radius:var(--radius);padding:12px;max-height:300px;overflow-y:auto;font-family:monospace;font-size:12px;color:var(--text2);line-height:1.8;margin-top:8px}
.log-box div{padding:2px 0}
</style>
</head>
<body>
<div class="sidebar">
  <div class="sidebar-header"><h2>◇ FreateOJ Wiki</h2><p id="sidebarInfo">Loading...</p></div>
  <nav class="sidebar-nav" id="sidebarNav"></nav>
</div>
<div class="main">
  <div id="app"></div>
</div>
<div class="toast" id="toast"></div>

<script>
const API = '/admin';

// ─── Router ─────────────────────────────────────────────────────────────────
function route() {
  const path = location.hash.slice(1) || '/';
  loadDash(path);
}

window.addEventListener('hashchange', route);

// ─── Toast ───────────────────────────────────────────────────────────────────
function showToast(msg, type = 'info') {
  const el = document.getElementById('toast');
  const d = document.createElement('div');
  d.className = 'toast-inner ' + type;
  d.innerHTML = '<div class="msg">' + msg + '</div>';
  el.appendChild(d);
  requestAnimationFrame(() => d.classList.add('show'));
  setTimeout(() => { d.classList.remove('show'); setTimeout(() => d.remove(), 300); }, 3000);
}

// ─── API ─────────────────────────────────────────────────────────────────────
async function api(url, opts = {}) {
  try {
    const r = await fetch(API + url, opts);
    const text = await r.text();
    try { return JSON.parse(text); } catch { return text; }
  } catch(e) { showToast('Network error: ' + e, 'error'); throw e; }
}

// ─── Sidebar ─────────────────────────────────────────────────────────────────
async function refreshSidebar() {
  const data = await api('/data');
  const info = document.getElementById('sidebarInfo');
  info.textContent = data.pages.length + ' pages · ' + data.sections.length + ' sections';

  const nav = document.getElementById('sidebarNav');
  let html = '<button class="nav-item active" onclick="location.hash=\'#/\'">📊 Dashboard</button>';
  html += '<button class="nav-section">Pages</button>';

  for (const sec of data.sections) {
    const pages = data.pages.filter(p => p.section_id === sec.id);
    html += '<div style="padding:8px 12px 2px;color:var(--text3);font-size:11px">📁 ' + sec.title + ' <span style="float:right">' + pages.length + '</span></div>';
    for (const p of pages) {
      const active = location.hash === '#/edit/' + p.id || location.hash === '#/page-seo/' + p.id ? 'active' : '';
      html += '<button class="nav-sub ' + active + '" onclick="location.hash=\'#/edit/' + p.id + '\'">' + p.title + '</button>';
    }
  }

  html += '<button class="nav-section">Settings</button>';
  html += '<button class="nav-item" onclick="location.hash=\'#/global-seo\'">🌐 Global SEO</button>';

  nav.innerHTML = html;
}

// ─── Dashboard ───────────────────────────────────────────────────────────────
async function loadDash(path) {
  await refreshSidebar();

  if (path === '/') { renderDash(); }
  else if (path.startsWith('/edit/')) { renderEditPage(path.split('/')[2]); }
  else if (path.startsWith('/page-seo/')) { renderPageSeo(path.split('/')[2]); }
  else if (path === '/global-seo') { renderGlobalSeo(); }
  else if (path === '/build') { renderBuild(); }
  else { renderDash(); }
}

async function renderDash() {
  const data = await api('/data');
  const tp = data.pages.length, ts = data.sections.length;
  const tw = data.pages.reduce((s, p) => s + (p.content.split(/\s+/).filter(Boolean).length), 0);
  const rt = Math.max(1, Math.floor(tw / 200));

  let html = '<h1>📊 Dashboard</h1>';
  html += '<div class="row">';
  html += statCard('📄', 'Pages', tp); html += statCard('📁', 'Sections', ts);
  html += statCard('📝', 'Words', tw); html += statCard('⏱', 'Read Time', rt + ' min');
  html += '</div>';

  html += '<h2>Quick Actions</h2>';
  html += '<div class="btn-group">';
  html += '<button class="btn btn-primary" onclick="startBuild()">🔨 Build Site</button>';
  html += '<button class="btn btn-success" onclick="location.hash=\'#/build\'">📋 Build Log</button>';
  html += '</div>';

  html += '<h2>Pages &amp; Sections</h2>';
  html += '<div class="card">';
  html += '<div style="display:flex;gap:8px;margin-bottom:12px">';
  html += '<input id="newSection" placeholder="New section name..." style="flex:1" onkeydown="if(event.key==\'Enter\')addSection()">';
  html += '<button class="btn btn-primary btn-sm" onclick="addSection()">+ Section</button>';
  html += '</div>';
  html += '<div style="display:flex;gap:8px;margin-bottom:12px">';
  html += '<select id="addPageSection" style="flex:1">';
  for (const s of data.sections) html += '<option value="' + s.id + '">' + s.title + '</option>';
  html += '</select>';
  html += '<input id="addPageTitle" placeholder="Page title..." style="flex:2" onkeydown="if(event.key==\'Enter\')addPage()">';
  html += '<input id="addPageSlug" placeholder="slug" style="flex:1" onkeydown="if(event.key==\'Enter\')addPage()">';
  html += '<button class="btn btn-success btn-sm" onclick="addPage()">+ Page</button>';
  html += '</div>';

  // Page list by section
  for (const sec of data.sections) {
    const pages = data.pages.filter(p => p.section_id === sec.id);
    html += '<div class="section-title">📁 ' + escHtml(sec.title) + ' <span style="float:right">';
    html += '<button class="btn btn-ghost" onclick="moveSection(' + sec.id + ",'up'" + ')">↑</button>';
    html += '<button class="btn btn-ghost" onclick="moveSection(' + sec.id + ",'down'" + ')">↓</button>';
    html += '<button class="btn btn-ghost" style="color:var(--danger)" onclick="deleteSection(' + sec.id + ',\'' + escHtml(sec.title) + '\')">✕</button>';
    html += '</span></div>';
    for (const p of pages) {
      html += '<div class="page-row">';
      html += '<span class="title">' + escHtml(p.title) + '</span>';
      html += '<span class="actions">';
      html += '<button class="btn btn-ghost" onclick="movePage(' + p.id + ",'up'" + ')">↑</button>';
      html += '<button class="btn btn-ghost" onclick="movePage(' + p.id + ",'down'" + ')">↓</button>';
      html += '<button class="btn btn-ghost" onclick="location.hash=\'#/edit/' + p.id + '\'">✏</button>';
      html += '<button class="btn btn-ghost" onclick="location.hash=\'#/page-seo/' + p.id + '\'">⚙</button>';
      html += '<button class="btn btn-ghost" onclick="duplicatePage(' + p.id + ')">⊕</button>';
      html += '<button class="btn btn-ghost" style="color:var(--danger)" onclick="deletePage(' + p.id + ',\'' + escHtml(p.title) + '\')">✕</button>';
      html += '</span></div>';
    }
  }
  html += '</div>';

  document.getElementById('app').innerHTML = html;
}

function statCard(icon, label, val) {
  return '<div class="stat card"><h3>' + icon + ' ' + label + '</h3><div class="val">' + val + '</div></div>';
}

function escHtml(s) { return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;'); }

// ─── Add / Delete / Move ─────────────────────────────────────────────────────
async function addSection() {
  const name = document.getElementById('newSection').value.trim();
  if (!name) return;
  await api('/section/add', { method:'POST', body: new URLSearchParams({title:name}) });
  showToast('Section added', 'success');
  document.getElementById('newSection').value = '';
  location.hash = '#/';
}

async function addPage() {
  const sid = document.getElementById('addPageSection').value;
  const title = document.getElementById('addPageTitle').value.trim();
  const slug = document.getElementById('addPageSlug').value.trim();
  if (!title || !slug) { showToast('Title and slug required', 'error'); return; }
  await api('/page/add', { method:'POST', body: new URLSearchParams({section_id:sid, title, slug}) });
  showToast('Page added', 'success');
  document.getElementById('addPageTitle').value = '';
  document.getElementById('addPageSlug').value = '';
  location.hash = '#/';
}

async function deleteSection(id, name) {
  if (!confirm('Delete section "' + name + '" and all its pages?')) return;
  await api('/section/delete/' + id, { method:'POST' });
  showToast('Deleted section', 'success');
  location.hash = '#/';
}

async function deletePage(id, name) {
  if (!confirm('Delete "' + name + '"?')) return;
  await api('/page/delete/' + id, { method:'POST' });
  showToast('Page deleted', 'success');
  location.hash = '#/';
}

async function moveSection(id, dir) {
  await api('/section/move/' + id + '/' + dir, { method:'POST' });
  location.hash = '#/';
}

async function movePage(id, dir) {
  await api('/page/move/' + id + '/' + dir, { method:'POST' });
  location.hash = '#/';
}

async function duplicatePage(id) {
  await api('/page/duplicate/' + id, { method:'POST' });
  showToast('Page duplicated', 'success');
  location.hash = '#/';
}

// ─── Edit Page ───────────────────────────────────────────────────────────────
async function renderEditPage(id) {
  const data = await api('/data');
  const page = data.pages.find(p => p.id == id);
  if (!page) { document.getElementById('app').innerHTML = '<h1>Page not found</h1>'; return; }

  let html = '<h1>✏ Edit: ' + escHtml(page.title) + '</h1>';
  html += '<div class="card">';

  html += '<div class="form-group"><label>Slug</label>';
  html += '<input id="editSlug" value="' + escHtml(page.slug) + '"></div>';

  html += '<div class="form-group"><label>Title</label>';
  html += '<input id="editTitle" value="' + escHtml(page.title) + '"></div>';

  html += '<div class="form-group"><label>Description</label>';
  html += '<input id="editDesc" value="' + escHtml(page.description) + '"></div>';

  html += '<div class="form-group"><label>Keywords</label>';
  html += '<input id="editKeywords" value="' + escHtml(page.keywords) + '"></div>';

  html += '<div class="form-group"><label>Content (Markdown)</label>';
  html += '<textarea id="editContent">' + escHtml(page.content) + '</textarea></div>';

  html += '<div class="btn-group">';
  html += '<button class="btn btn-primary" onclick="savePage(' + id + ')">💾 Save</button>';
  html += '<button class="btn btn-ghost" onclick="previewPage()">👁 Preview</button>';
  html += '</div>';

  html += '<div id="previewArea" style="display:none;margin-top:12px">';
  html += '<label>Preview</label>';
  html += '<div class="log-box" id="previewContent"></div>';
  html += '</div>';

  html += '</div>';
  document.getElementById('app').innerHTML = html;
}

async function savePage(id) {
  const body = new URLSearchParams({
    slug: document.getElementById('editSlug').value,
    title: document.getElementById('editTitle').value,
    description: document.getElementById('editDesc').value,
    keywords: document.getElementById('editKeywords').value,
    content: document.getElementById('editContent').value,
  });
  await api('/page/update/' + id, { method:'POST', body });
  showToast('Page saved', 'success');
  location.hash = '#/edit/' + id;
}

async function previewPage() {
  const md = document.getElementById('editContent').value;
  const r = await api('/preview', { method:'POST', body: new URLSearchParams({content: md}) });
  document.getElementById('previewArea').style.display = 'block';
  document.getElementById('previewContent').textContent = r.html || 'Error rendering';
}

// ─── Page SEO ────────────────────────────────────────────────────────────────
async function renderPageSeo(id) {
  const data = await api('/data');
  const page = data.pages.find(p => p.id == id);
  if (!page) { document.getElementById('app').innerHTML = '<h1>Page not found</h1>'; return; }

  let html = '<h1>⚙ SEO: ' + escHtml(page.title) + '</h1>';
  html += '<div class="card">';

  const fields = [
    ['seo_og_image','OG Image URL'], ['seo_og_width','OG Image Width'],
    ['seo_og_height','OG Image Height'], ['seo_twitter_site','Twitter Site'],
    ['seo_twitter_card','Twitter Card'], ['seo_robots','Robots'],
  ];
  for (const [key, label] of fields) {
    html += '<div class="form-group"><label>' + label + '</label>';
    html += '<input id="' + key + '" value="' + escHtml(page[key]) + '"></div>';
  }

  html += '<button class="btn btn-primary" onclick="savePageSeo(' + id + ')">💾 Save SEO</button>';
  html += '</div>';
  document.getElementById('app').innerHTML = html;
}

async function savePageSeo(id) {
  const fields = ['seo_og_image','seo_og_width','seo_og_height','seo_twitter_site','seo_twitter_card','seo_robots'];
  const body = new URLSearchParams();
  for (const f of fields) body.append(f, document.getElementById(f).value);
  await api('/page/update/' + id, { method:'POST', body });
  showToast('SEO saved', 'success');
}

// ─── Global SEO ──────────────────────────────────────────────────────────────
async function renderGlobalSeo() {
  const data = await api('/data');
  const s = data.settings;

  let html = '<h1>🌐 Global SEO Settings</h1>';
  html += '<div class="card">';

  const fields = [
    ['SITE_NAME','Site Name'],['SITE_URL','Site URL'],['SITE_DESCRIPTION','Description'],
    ['SITE_AUTHOR','Author'],['DEFAULT_LANG','Language'],['DEFAULT_LOCALE','Locale'],
    ['OG_IMAGE','OG Image'],['OG_IMAGE_WIDTH','OG Width'],['OG_IMAGE_HEIGHT','OG Height'],
    ['TWITTER_SITE','Twitter'],['TWITTER_CARD','Card'],['ROBOTS','Robots'],
    ['FAVICON_SVG','Favicon SVG Path'],
  ];
  for (const [key, label] of fields) {
    const val = s[key] || '';
    html += '<div class="form-group"><label>' + label + '</label>';
    html += '<input id="gs-' + key + '" value="' + escHtml(val) + '"></div>';
  }

  html += '<button class="btn btn-primary" onclick="saveGlobalSeo()">💾 Save</button>';
  html += '</div>';
  document.getElementById('app').innerHTML = html;
}

async function saveGlobalSeo() {
  const keys = ['SITE_NAME','SITE_URL','SITE_DESCRIPTION','SITE_AUTHOR','DEFAULT_LANG','DEFAULT_LOCALE',
    'OG_IMAGE','OG_IMAGE_WIDTH','OG_IMAGE_HEIGHT','TWITTER_SITE','TWITTER_CARD','ROBOTS','FAVICON_SVG'];
  const body = new URLSearchParams();
  for (const k of keys) body.append(k, document.getElementById('gs-' + k).value);
  await api('/settings', { method:'POST', body });
  showToast('Settings saved', 'success');
  location.hash = '#/global-seo';
}

// ─── Build ───────────────────────────────────────────────────────────────────
async function renderBuild() {
  let html = '<h1>📋 Build</h1>';
  html += '<div class="card">';
  html += '<button class="btn btn-primary" onclick="startBuild()">🔨 Build Site</button>';
  html += '<div class="progress" style="margin-top:12px"><div class="progress-inner" id="buildProgress"></div></div>';
  html += '<div class="progress-text" id="buildStatus">Ready</div>';
  html += '<div class="log-box" id="buildLog"></div>';
  html += '</div>';
  document.getElementById('app').innerHTML = html;
}

async function startBuild() {
  const progress = document.getElementById('buildProgress');
  const status = document.getElementById('buildStatus');
  const log = document.getElementById('buildLog');
  if (!log) { showToast('Go to Build tab first', 'info'); return; }

  log.innerHTML = '';
  status.textContent = 'Building...';
  progress.style.width = '10%';

  // EventSource would be cleaner but SSE needs async. Poll instead.
  const r = await api('/build', { method:'POST' });
  if (r.ok) {
    progress.style.width = '100%';
    status.textContent = 'Build complete!';
    showToast('Build complete!', 'success');
    if (r.logs) log.innerHTML = r.logs.map(l => '<div>' + escHtml(l) + '</div>').join('');
  } else {
    status.textContent = 'Build failed';
    showToast('Build failed', 'error');
    if (r.logs) log.innerHTML = r.logs.map(l => '<div>' + escHtml(l) + '</div>').join('');
  }
}

// ─── Init ────────────────────────────────────────────────────────────────────
route();
</script>
</body>
</html>"#
}

// ─── Server ──────────────────────────────────────────────────────────────────

pub fn serve(db_path: &str, build_dir: &str, port: u16) -> Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    let server = Server::http(&addr)
        .map_err(|e| anyhow::anyhow!("Failed to start server: {}", e))?;
    let db = db_path.to_string();
    let out = build_dir.to_string();

    println!("◇ FreateOJ Wiki Admin");
    println!("   Server:  http://localhost:{}", port);
    println!("   Admin:   http://localhost:{}/admin", port);
    println!("   Press Ctrl+C to stop");

    for mut request in server.incoming_requests() {
        let url = request.url().to_string();
        let method = request.method().as_str().to_string();

        // ── Parse URL ───────────────────────────────────────────────────────
        let path = url.split('?').next().unwrap_or(&url).to_string();

        // Read body if POST
        let mut body = String::new();
        if method == "POST" {
            let reader = request.as_reader();
            let mut buf = Vec::new();
            let _ = reader.read_to_end(&mut buf);
            body = String::from_utf8_lossy(&buf).to_string();
        }

        let response = match handle(&path, &method, &body, &db, &out) {
            Ok(r) => r,
            Err(e) => Response::from_string(format!("Error: {}", e)).with_status_code(500),
        };

        if let Err(e) = request.respond(response) {
            eprintln!("Response error: {}", e);
        }
    }

    Ok(())
}

fn handle(path: &str, _method: &str, body: &str, db_path: &str, build_dir: &str) -> Result<Response<std::io::Cursor<Vec<u8>>>> {
    // ── Admin routes ─────────────────────────────────────────────────────────
    if path == "/admin" || path == "/admin/" {
        return Ok(ok(admin_html(), "text/html"));
    }

    // JSON data endpoint
    if path == "/admin/data" {
        let conn = db::get_conn(db_path)?;
        let settings = db::get_all_settings(&conn).unwrap_or_default();
        let sections = db::get_all_sections(&conn).unwrap_or_default();
        let pages = db::get_all_pages(&conn).unwrap_or_default();

        let mut pages_json = Vec::new();
        for p in &pages {
            pages_json.push(format!(
                r#"{{"id":{},"section_id":{},"slug":"{}","title":"{}","description":"{}","keywords":"{}","content":{}, "seo_og_image":"{}","seo_og_width":"{}","seo_og_height":"{}","seo_twitter_site":"{}","seo_twitter_card":"{}","seo_robots":"{}"}}"#,
                p.id, p.section_id, json_esc(&p.slug), json_esc(&p.title), json_esc(&p.description),
                json_esc(&p.keywords), json_esc(&p.content), json_esc(&p.seo_og_image),
                json_esc(&p.seo_og_width), json_esc(&p.seo_og_height), json_esc(&p.seo_twitter_site),
                json_esc(&p.seo_twitter_card), json_esc(&p.seo_robots),
            ));
        }

        let mut sec_json = Vec::new();
        for s in &sections {
            sec_json.push(format!(r#"{{"id":{},"title":"{}"}}"#, s.id, json_esc(&s.title)));
        }

        let mut set_json = Vec::new();
        for (k, v) in &settings {
            set_json.push(format!(r#""{}":"{}""#, json_esc(k), json_esc(v)));
        }

        let json = format!(
            r#"{{"pages":[{}],"sections":[{}],"settings":{{{}}}}}"#,
            pages_json.join(","), sec_json.join(","), set_json.join(",")
        );
        return Ok(ok_json(&json));
    }

    // Build
    if path == "/admin/build" {
        let mut logs = vec!["Preparing build...".to_string()];
        let conn = db::get_conn(db_path)?;
        let settings = db::get_all_settings(&conn).unwrap_or_default();
        let sections = db::get_all_sections(&conn).unwrap_or_default();
        let pages = db::get_all_pages(&conn).unwrap_or_default();

        match build::build(&settings, &sections, &pages, build_dir) {
            Ok(_) => {
                logs.push("Build complete!".to_string());
                let logs_json: Vec<String> = logs.iter().map(|l| format!(r#""{}""#, json_esc(l))).collect();
                return Ok(ok_json(&format!(r#"{{"ok":true,"logs":[{}]}}"#, logs_json.join(","))));
            }
            Err(e) => {
                logs.push(format!("Error: {}", e));
                let logs_json: Vec<String> = logs.iter().map(|l| format!(r#""{}""#, json_esc(l))).collect();
                return Ok(ok_json(&format!(r#"{{"ok":false,"logs":[{}]}}"#, logs_json.join(","))));
            }
        }
    }

    // Preview markdown
    if path == "/admin/preview" {
        let params: Vec<&str> = body.split('&').collect();
        let mut content = "";
        for p in &params {
            if let Some(val) = p.strip_prefix("content=") {
                content = val;
            }
        }
        let decoded = url_decode(content);
        let html = build::render_markdown(&decoded);
        return Ok(ok_json(&format!(r#"{{"html":"{}"}}"#, json_esc(&html))));
    }

    // ── CRUD: Sections ──────────────────────────────────────────────────────
    if path == "/admin/section/add" {
        let params = parse_params(body);
        let title = params.get("title").cloned().unwrap_or_default();
        let conn = db::get_conn(db_path)?;
        db::add_section(&conn, &title)?;
        return Ok(ok_json(r#"{"ok":true}"#));
    }

    if let Some(id_str) = path.strip_prefix("/admin/section/delete/") {
        let id: i64 = id_str.parse()?;
        let conn = db::get_conn(db_path)?;
        db::delete_section(&conn, id)?;
        return Ok(ok_json(r#"{"ok":true}"#));
    }

    if let Some(rest) = path.strip_prefix("/admin/section/move/") {
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() == 2 {
            let id: i64 = parts[0].parse()?;
            let conn = db::get_conn(db_path)?;
            db::move_section(&conn, id, parts[1])?;
            return Ok(ok_json(r#"{"ok":true}"#));
        }
    }

    // ── CRUD: Pages ─────────────────────────────────────────────────────────
    if path == "/admin/page/add" {
        let params = parse_params(body);
        let section_id: i64 = params.get("section_id").and_then(|s| s.parse().ok()).unwrap_or(0);
        let slug = params.get("slug").cloned().unwrap_or_default();
        let title = params.get("title").cloned().unwrap_or_default();
        let conn = db::get_conn(db_path)?;
        db::add_page(&conn, section_id, &slug, &title)?;
        return Ok(ok_json(r#"{"ok":true}"#));
    }

    if let Some(rest) = path.strip_prefix("/admin/page/update/") {
        let id: i64 = rest.parse()?;
        let params = parse_params(body);
        let slug = params.get("slug").cloned().unwrap_or_default();
        let title = params.get("title").cloned().unwrap_or_default();
        let description = params.get("description").cloned().unwrap_or_default();
        let keywords = params.get("keywords").cloned().unwrap_or_default();
        let content = params.get("content").cloned().unwrap_or_default();
        let seo_og_image = params.get("seo_og_image").cloned().unwrap_or_default();
        let seo_og_width = params.get("seo_og_width").cloned().unwrap_or_default();
        let seo_og_height = params.get("seo_og_height").cloned().unwrap_or_default();
        let seo_twitter_site = params.get("seo_twitter_site").cloned().unwrap_or_default();
        let seo_twitter_card = params.get("seo_twitter_card").cloned().unwrap_or_default();
        let seo_robots = params.get("seo_robots").cloned().unwrap_or_default();
        let conn = db::get_conn(db_path)?;
        db::update_page(&conn, id, &slug, &title, &description, &keywords, &content,
            &seo_og_image, &seo_og_width, &seo_og_height, &seo_twitter_site, &seo_twitter_card, &seo_robots)?;
        return Ok(ok_json(r#"{"ok":true}"#));
    }

    if let Some(id_str) = path.strip_prefix("/admin/page/delete/") {
        let id: i64 = id_str.parse()?;
        let conn = db::get_conn(db_path)?;
        db::delete_page(&conn, id)?;
        return Ok(ok_json(r#"{"ok":true}"#));
    }

    if let Some(rest) = path.strip_prefix("/admin/page/move/") {
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() == 2 {
            let id: i64 = parts[0].parse()?;
            let conn = db::get_conn(db_path)?;
            db::move_page(&conn, id, parts[1])?;
            return Ok(ok_json(r#"{"ok":true}"#));
        }
    }

    if let Some(id_str) = path.strip_prefix("/admin/page/duplicate/") {
        let id: i64 = id_str.parse()?;
        let conn = db::get_conn(db_path)?;
        db::duplicate_page(&conn, id)?;
        return Ok(ok_json(r#"{"ok":true}"#));
    }

    // ── Global settings ─────────────────────────────────────────────────────
    if path == "/admin/settings" {
        let params = parse_params(body);
        let conn = db::get_conn(db_path)?;
        for (k, v) in &params {
            db::set_setting(&conn, k, v)?;
        }
        return Ok(ok_json(r#"{"ok":true}"#));
    }

    // ── Serve static files ──────────────────────────────────────────────────
    let file_url = if path == "/" { "/index.html" } else { &path };
    let file_path = Path::new(build_dir).join(file_url.trim_start_matches('/'));

    if file_path.exists() && file_path.is_file() {
        let mut file = fs::File::open(&file_path)?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;

        let ct = mime(file_url);
        let response = Response::from_data(content)
            .with_header(Header::from_bytes(b"Content-Type", format!("{}; charset=utf-8", ct).as_bytes()).unwrap());
        return Ok(response);
    }

    // ── Fallback: redirect /admin ───────────────────────────────────────────
    if file_url == "/index.html" {
        return Ok(redirect("/admin"));
    }

    Ok(not_found())
}

// ─── URL param parsing ───────────────────────────────────────────────────────

fn parse_params(body: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for pair in body.split('&') {
        if let Some(eq) = pair.find('=') {
            let key = url_decode(&pair[..eq]);
            let val = url_decode(&pair[eq+1..]);
            map.insert(key, val);
        }
    }
    map
}

fn url_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '+' {
            result.push(' ');
        } else if c == '%' {
            let hi = chars.next().and_then(|c| c.to_digit(16)).unwrap_or(0);
            let lo = chars.next().and_then(|c| c.to_digit(16)).unwrap_or(0);
            result.push(char::from((hi * 16 + lo) as u8));
        } else {
            result.push(c);
        }
    }
    result
}

fn json_esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}
