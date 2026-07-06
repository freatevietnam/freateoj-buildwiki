use anyhow::Result;
use std::fs;
use std::io::Read;
use std::path::Path;
use tiny_http::{Server, Response, Header};

use crate::build;
use crate::db;

fn mime(path: &str) -> &str {
    match path.rsplit('.').next().unwrap_or("") {
        "html"|"htm" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "xml"|"xsl" => "application/xml",
        "png" => "image/png",
        "jpg"|"jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "webp" => "image/webp",
        "woff"|"woff2" => "font/woff", "ttf" => "font/ttf", "otf" => "font/otf",
        "txt" => "text/plain",
        "map" => "application/json",
        _ => "application/octet-stream",
    }
}

fn ok(data: &str, ct: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(data).with_header(
        Header::from_bytes(b"Content-Type", format!("{}; charset=utf-8", ct).as_bytes()).unwrap(),
    )
}

fn ok_json(data: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    ok(data, "application/json")
}

fn not_found() -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string("404").with_status_code(404)
}

// ─── Admin HTML (embedded SPA) ───────────────────────────────────────────────

fn admin_html() -> &'static str {
    r#"<!DOCTYPE html>
<html lang="vi">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1.0">
<title>Wiki Builder</title>
<style>
*{margin:0;padding:0;box-sizing:border-box}
:root{--bg:#0d0d1a;--bg2:#151528;--surface:#1a1a30;--hover:#28284f;--active:#33336b;--text:#d0d0e0;--text2:#9090b0;--text3:#585880;--accent:#7c7cf0;--success:#22c55e;--danger:#ef4444;--border:#2a2a4a;--radius:6px}
body{font-family:'Inter',system-ui,-apple-system,sans-serif;background:var(--bg);color:var(--text);display:flex;min-height:100vh;font-size:14px;line-height:1.6}
/* ── Sidebar ── */
.sidebar{width:260px;background:var(--bg2);border-right:1px solid var(--border);height:100vh;position:sticky;top:0;overflow-y:auto;flex-shrink:0}
.sidebar-header{padding:20px 16px;border-bottom:1px solid var(--border)}
.sidebar-header h2{color:var(--accent);font-size:15px;font-weight:700}
.sidebar-header p{color:var(--text3);font-size:11px;margin-top:2px}
.nav-item,.nav-section{display:block;width:100%;padding:9px 16px;background:none;border:none;color:var(--text2);font-size:13px;cursor:pointer;text-align:left;transition:all .12s}
.nav-item:hover{background:var(--hover);color:var(--text)}
.nav-item.active{color:var(--accent);background:var(--active)}
.nav-section{padding:14px 16px 4px;color:var(--text3);font-size:10px;text-transform:uppercase;letter-spacing:1.2px;font-weight:700}
.nav-page{padding-left:28px;font-size:12px}
.badge{float:right;background:var(--active);color:var(--accent);border-radius:10px;padding:1px 7px;font-size:10px}
/* ── Main ── */
.main{flex:1;padding:24px;max-width:1100px;width:100%}
h1{font-size:20px;margin-bottom:16px;font-weight:700}
.card{background:var(--surface);border:1px solid var(--border);border-radius:var(--radius);padding:20px;margin-bottom:16px}
label{display:block;color:var(--text2);font-size:12px;margin-bottom:3px;font-weight:600}
input,textarea{width:100%;padding:9px 12px;background:var(--bg);border:1px solid var(--border);border-radius:var(--radius);color:var(--text);font-size:13px;outline:none;transition:border-color .15s}
input:focus,textarea:focus{border-color:var(--accent)}
textarea{font-family:monospace;font-size:12px;min-height:280px;resize:vertical}
.btn{display:inline-block;padding:9px 18px;border:none;border-radius:var(--radius);font-size:13px;font-weight:600;cursor:pointer;transition:opacity .15s;text-decoration:none}
.btn-primary{background:var(--accent);color:#fff}
.btn-primary:hover{opacity:.85}
.btn-success{background:var(--success);color:#fff}
.btn-danger{background:var(--danger);color:#fff}
.btn-ghost{background:transparent;color:var(--text2);padding:4px 8px;font-size:12px}
.btn-ghost:hover{color:var(--text)}
.btn-sm{padding:6px 14px;font-size:12px}
.flex{display:flex;gap:8px;flex-wrap:wrap;align-items:center}
.mt{margin-top:12px}
.mb{margin-bottom:8px}
/* ── Page list ── */
.prow{display:flex;align-items:center;justify-content:space-between;padding:6px 10px;border-radius:var(--radius);margin:1px 0}
.prow:hover{background:var(--hover)}
.prow .title{color:var(--text);font-weight:500}
.prow .act{display:flex;gap:3px;opacity:0;transition:opacity .12s}
.prow:hover .act{opacity:1}
.psec{padding:14px 10px 4px;color:var(--text3);font-size:10px;text-transform:uppercase;letter-spacing:1px;font-weight:700}
/* ── Build log ── */
.log{background:#0a0a14;border:1px solid var(--border);border-radius:var(--radius);padding:12px;max-height:280px;overflow-y:auto;font-family:monospace;font-size:12px;color:var(--text2);line-height:1.9;margin-top:8px}
.prog{height:5px;background:var(--bg);border-radius:3px;overflow:hidden;margin:8px 0}
.prog-in{height:100%;background:var(--accent);border-radius:3px;transition:width .3s;width:0%}
/* ── Tab layout ── */
.tabs{display:flex;gap:0;margin-bottom:14px;border-bottom:1px solid var(--border)}
.tab{padding:9px 18px;border:none;background:none;color:var(--text2);cursor:pointer;font-size:13px;border-bottom:2px solid transparent;transition:all .12s}
.tab:hover{color:var(--text)}
.tab.act{color:var(--accent);border-bottom-color:var(--accent)}
.tc{display:none}
.tc.act{display:block}
/* ── Toast ── */
#toast{position:fixed;top:16px;right:16px;z-index:999}
.ti{background:var(--surface);border:1px solid var(--border);border-radius:var(--radius);padding:12px 18px;margin-bottom:6px;box-shadow:0 4px 20px #0006;transform:translateX(120%);transition:transform .25s;max-width:340px;font-size:13px}
.ti.s{transform:translateX(0)}
.ti.suc{border-left:3px solid var(--success)}
.ti.err{border-left:3px solid var(--danger)}
.ti.inf{border-left:3px solid var(--accent)}
</style>
</head>
<body>
<div class="sidebar" id="sidebar"></div>
<div class="main"><div id="app"></div></div>
<div id="toast"></div>
<script>
const A='/api'; let D=null;

function $(id){return document.getElementById(id)}
function esc(s){return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;')}
function toast(m,t){const e=document.getElementById('toast');const d=document.createElement('div');d.className='ti '+t;d.innerHTML=m;e.appendChild(d);requestAnimationFrame(()=>d.classList.add('s'));setTimeout(()=>{d.classList.remove('s');setTimeout(()=>d.remove(),250)},3000)}

async function api(u,o={}){try{const r=await fetch(A+u,o);const t=await r.text();try{return JSON.parse(t)}catch{return t}}catch(e){toast('Lỗi: '+e,'err');throw e}}

async function load(){D=await api('/data');sidebar();route()}

function sidebar(){
  const s=$('sidebar');let h='<div class="sidebar-header"><h2>◇ Wiki Builder</h2><p>'+D.pages.length+' pages</p></div>';
  h+='<button class="nav-item act" onclick="go(\'/\')">📊 Dashboard</button>';
  h+='<button class="nav-section">Pages</button>';
  for(const sec of D.sections){
    const pp=D.pages.filter(p=>p.section_id===sec.id);
    h+='<button class="nav-section" style="padding-bottom:2px">📁 '+esc(sec.title)+' <span class="badge">'+pp.length+'</span></button>';
    for(const p of pp){
      const a=location.hash==='#/edit/'+p.id?'active':'';
      h+='<button class="nav-item nav-page '+a+'" onclick="go(\'/edit/'+p.id+'\')">'+esc(p.title)+'</button>'
    }
  }
  h+='<button class="nav-section" style="padding-bottom:6px;padding-top:16px">Settings</button>';
  h+='<button class="nav-item" onclick="go(\'/seo\')">🌐 Global SEO</button>';
  s.innerHTML=h
}

function go(p){location.hash='#'+p}
window.addEventListener('hashchange',()=>load());

function route(){
  const p=location.hash.slice(1)||'/';
  if(p==='/')dash();
  else if(p.startsWith('/edit/'))edit(p.split('/')[2]);
  else if(p==='/seo')seo();
  else dash()
}

// ── Dashboard ──
function dash(){
  const tp=D.pages.length;
  const tw=D.pages.reduce((s,p)=>s+(p.content.split(/\s+/).filter(Boolean).length),0);
  const rt=Math.max(1,Math.floor(tw/200));

  let h='<h1>📊 Dashboard</h1><div class="flex mb">';
  h+=`<div class="card" style="flex:1;min-width:120px"><div style="font-size:28px;font-weight:700">${tp}</div><div style="color:var(--text3);font-size:12px">Pages</div></div>`;
  h+=`<div class="card" style="flex:1;min-width:120px"><div style="font-size:28px;font-weight:700">${tw}</div><div style="color:var(--text3);font-size:12px">Words</div></div>`;
  h+=`<div class="card" style="flex:1;min-width:120px"><div style="font-size:28px;font-weight:700">${rt}m</div><div style="color:var(--text3);font-size:12px">Read time</div></div>`;
  h+='</div>';
  h+='<button class="btn btn-primary" onclick="bld()">🔨 Build Site</button>';
  h+=' <button class="btn btn-ghost" onclick="go(\'/build\')">📋 Log</button>';

  h+='<div class="card mt"><div class="flex mb">';
  h+='<input id="ns" placeholder="Section name..." style="flex:1" onkeydown="if(event.key==\'Enter\')addSec()">';
  h+='<button class="btn btn-primary btn-sm" onclick="addSec()">+ Section</button></div>';
  h+='<div class="flex mb">';
  h+='<select id="aps" style="flex:1">';for(const s of D.sections)h+='<option value='+s.id+'>'+esc(s.title)+'</option>';h+='</select>';
  h+='<input id="apt" placeholder="Title" style="flex:2" onkeydown="if(event.key==\'Enter\')addPg()">';
  h+='<input id="apsl" placeholder="slug" style="flex:1" onkeydown="if(event.key==\'Enter\')addPg()">';
  h+='<button class="btn btn-success btn-sm" onclick="addPg()">+ Page</button></div>';

  for(const sec of D.sections){
    const pp=D.pages.filter(p=>p.section_id===sec.id);
    h+='<div class="psec">📁 '+esc(sec.title)+' <span style="float:right">';
    h+='<button class="btn btn-ghost" onclick="mvSec('+sec.id+",'up')"+'">↑</button>';
    h+='<button class="btn btn-ghost" onclick="mvSec('+sec.id+",'down')"+'">↓</button>';
    h+='<button class="btn btn-ghost" style="color:var(--danger)" onclick="delSec('+sec.id+')">✕</button></span></div>';
    for(const p of pp){
      h+='<div class="prow"><span class="title">'+esc(p.title)+'</span><span class="act">';
      h+='<button class="btn btn-ghost" onclick="mvPg('+p.id+",'up')"+'">↑</button>';
      h+='<button class="btn btn-ghost" onclick="mvPg('+p.id+",'down')"+'">↓</button>';
      h+='<button class="btn btn-ghost" onclick="go('/edit/'+p.id)">✏</button>';
      h+='<button class="btn btn-ghost" onclick="dupPg('+p.id+')">⊕</button>';
      h+='<button class="btn btn-ghost" style="color:var(--danger)" onclick="delPg('+p.id+')">✕</button>';
      h+='</span></div>'
    }
  }
  $('app').innerHTML=h
}

// ── CRUD ──
async function addSec(){
  const n=$('ns').value.trim();if(!n)return;
  await api('/section/add',{method:'POST',body:new URLSearchParams({title:n})});
  toast('Section added','suc');load()
}
async function addPg(){
  const sid=$('aps').value,t=$('apt').value.trim(),s=$('apsl').value.trim();
  if(!t||!s){toast('Title + slug required','err');return}
  await api('/page/add',{method:'POST',body:new URLSearchParams({section_id:sid,title:t,slug:s})});
  $('apt').value='';$('apsl').value='';toast('Page added','suc');load()
}
async function delSec(id){if(!confirm('Delete section?'))return;await api('/section/delete/'+id,{method:'POST'});toast('Deleted','suc');load()}
async function delPg(id){if(!confirm('Delete page?'))return;await api('/page/delete/'+id,{method:'POST'});toast('Deleted','suc');load()}
async function mvSec(id,d){await api('/section/move/'+id+'/'+d,{method:'POST'});load()}
async function mvPg(id,d){await api('/page/move/'+id+'/'+d,{method:'POST'});load()}
async function dupPg(id){await api('/page/duplicate/'+id,{method:'POST'});toast('Duplicated','suc');load()}

// ── Editor ──
function edit(id){
  const p=D.pages.find(x=>x.id==id);if(!p){$('app').innerHTML='<h1>Not found</h1>';return}
  let h='<h1>✏ '+esc(p.title)+'</h1><div class="card">';
  h+='<div class="flex mb">';
  h+='<div style="flex:1"><label>Slug</label><input id="es" value="'+esc(p.slug)+'"></div>';
  h+='<div style="flex:2"><label>Title</label><input id="et" value="'+esc(p.title)+'"></div></div>';
  h+='<div class="mb"><label>Description</label><input id="ed" value="'+esc(p.description)+'"></div>';
  h+='<div class="mb"><label>Keywords</label><input id="ek" value="'+esc(p.keywords)+'"></div>';
  h+='<div class="mb"><label>Markdown</label><textarea id="ec">'+esc(p.content)+'</textarea></div>';
  h+='<div class="flex"><button class="btn btn-primary" onclick="save('+id+')">💾 Save</button>';
  h+='<button class="btn btn-ghost" onclick="preview()">👁 Preview</button>';
  h+='<button class="btn btn-ghost" onclick="go(\'/seo-p/'+id+'\')">⚙ SEO</button></div>';
  h+='<div id="pv" style="display:none;margin-top:12px"><label>Preview</label><div class="log" id="pvx"></div></div></div>';
  $('app').innerHTML=h
}
async function save(id){
  const b=new URLSearchParams({slug:$('es').value,title:$('et').value,description:$('ed').value,keywords:$('ek').value,content:$('ec').value});
  await api('/page/update/'+id,{method:'POST',body:b});toast('Saved','suc');load()
}
async function preview(){
  const r=await api('/preview',{method:'POST',body:new URLSearchParams({content:$('ec').value})});
  $('pv').style.display='block';$('pvx').textContent=r.html||'Error'
}

// ── SEO ──
async function seo(){
  const s=D.settings;let h='<h1>🌐 Global SEO</h1><div class="card">';
  const f=[['SITE_NAME','Name'],['SITE_URL','URL'],['SITE_DESCRIPTION','Description'],['SITE_AUTHOR','Author'],
    ['DEFAULT_LANG','Lang'],['DEFAULT_LOCALE','Locale'],['OG_IMAGE','OG Image'],['OG_IMAGE_WIDTH','OG W'],
    ['OG_IMAGE_HEIGHT','OG H'],['TWITTER_SITE','Twitter'],['TWITTER_CARD','Card'],['ROBOTS','Robots'],['FAVICON_SVG','Favicon SVG']];
  for(const[k,l]of f)h+='<div class="mb"><label>'+l+'</label><input id="gs-'+k+'" value="'+esc(s[k]||'')+'"></div>';
  h+='<button class="btn btn-primary" onclick="saveSeo()">💾 Save</button></div>';
  $('app').innerHTML=h
}
async function saveSeo(){
  const k=['SITE_NAME','SITE_URL','SITE_DESCRIPTION','SITE_AUTHOR','DEFAULT_LANG','DEFAULT_LOCALE',
    'OG_IMAGE','OG_IMAGE_WIDTH','OG_IMAGE_HEIGHT','TWITTER_SITE','TWITTER_CARD','ROBOTS','FAVICON_SVG'];
  const b=new URLSearchParams();for(const x of k)b.append(x,$('gs-'+x).value);
  await api('/settings',{method:'POST',body:b});toast('Saved','suc');load()
}

// ── Build ──
async function bld(){
  const r=await api('/build',{method:'POST'});
  if(r.ok){toast('Build complete','suc');if(r.logs)$('bl').innerHTML=r.logs.map(l=>'<div>'+esc(l)+'</div>').join('');$('bp').style.width='100%'}
  else{toast('Build failed','err');if(r.logs)$('bl').innerHTML=r.logs.map(l=>'<div>'+esc(l)+'</div>').join('')}
}

load()
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

    // Initial build
    if let Ok(conn) = db::get_conn(&db) {
        if let (Ok(settings), Ok(sections), Ok(pages)) =
            (db::get_all_settings(&conn), db::get_all_sections(&conn), db::get_all_pages(&conn))
        {
            let _ = build::build(&settings, &sections, &pages, &out);
        }
    }

    println!("◇ Wiki Builder Dev Server");
    println!("   Editor: http://localhost:{}/admin", port);
    println!("   Site:   http://localhost:{}/", port);
    println!("   Press Ctrl+C to stop");

    for mut request in server.incoming_requests() {
        let url = request.url().to_string();
        let method = request.method().as_str().to_string();

        let mut body = String::new();
        if method == "POST" {
            let mut buf = Vec::new();
            let _ = request.as_reader().read_to_end(&mut buf);
            body = String::from_utf8_lossy(&buf).to_string();
        }

        let response = match handle(&url, &body, &db, &out) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("  Error: {}", e);
                Response::from_string(format!("Error: {}", e)).with_status_code(500)
            }
        };

        if let Err(e) = request.respond(response) {
            eprintln!("Response error: {}", e);
        }
    }

    Ok(())
}

fn handle(url: &str, body: &str, db_path: &str, build_dir: &str) -> Result<Response<std::io::Cursor<Vec<u8>>>> {
    let path = url.split('?').next().unwrap_or(url);

    // ── Admin UI ────────────────────────────────────────────────────────────
    if path == "/admin" || path == "/admin/" {
        return Ok(ok(admin_html(), "text/html"));
    }

    // ── API: data ───────────────────────────────────────────────────────────
    if path == "/api/data" {
        let conn = db::get_conn(db_path)?;
        let settings = db::get_all_settings(&conn)?;
        let sections = db::get_all_sections(&conn)?;
        let pages = db::get_all_pages(&conn)?;

        let pages_json: Vec<String> = pages.iter().map(|p| format!(
            r#"{{"id":{},"section_id":{},"slug":"{}","title":"{}","description":"{}","keywords":"{}","content":{},"seo_og_image":"{}","seo_og_width":"{}","seo_og_height":"{}","seo_twitter_site":"{}","seo_twitter_card":"{}","seo_robots":"{}"}}"#,
            p.id, p.section_id, jesc(&p.slug), jesc(&p.title), jesc(&p.description), jesc(&p.keywords),
            jesc(&p.content), jesc(&p.seo_og_image), jesc(&p.seo_og_width), jesc(&p.seo_og_height),
            jesc(&p.seo_twitter_site), jesc(&p.seo_twitter_card), jesc(&p.seo_robots),
        )).collect();

        let sections_json: Vec<String> = sections.iter().map(|s| format!(
            r#"{{"id":{},"title":"{}"}}"#, s.id, jesc(&s.title),
        )).collect();

        let settings_json: Vec<String> = settings.iter().map(|(k,v)| format!(
            r#""{}":"{}""#, jesc(k), jesc(v),
        )).collect();

        return Ok(ok_json(&format!(
            r#"{{"pages":[{}],"sections":[{}],"settings":{{{}}}}}"#,
            pages_json.join(","), sections_json.join(","), settings_json.join(",")
        )));
    }

    // ── API: build ──────────────────────────────────────────────────────────
    if path == "/api/build" {
        let mut logs = vec!["Preparing build...".to_string()];
        let conn = match db::get_conn(db_path) {
            Ok(c) => c, Err(e) => { logs.push(format!("DB error: {}", e)); return build_resp(&logs, false); }
        };
        let (settings, sections, pages) = (
            db::get_all_settings(&conn).unwrap_or_default(),
            db::get_all_sections(&conn).unwrap_or_default(),
            db::get_all_pages(&conn).unwrap_or_default(),
        );
        return match build::build(&settings, &sections, &pages, build_dir) {
            Ok(_) => { logs.push("Build complete!".to_string()); build_resp(&logs, true) }
            Err(e) => { logs.push(format!("Error: {}", e)); build_resp(&logs, false) }
        };
    }

    // ── API: preview ────────────────────────────────────────────────────────
    if path == "/api/preview" {
        let params = parse(body);
        let content = params.get("content").cloned().unwrap_or_default();
        let html = build::render_markdown(&content);
        return Ok(ok_json(&format!(r#"{{"html":"{}"}}"#, jesc(&html))));
    }

    // ── API: sections ───────────────────────────────────────────────────────
    if path == "/api/section/add" {
        let params = parse(body);
        let conn = db::get_conn(db_path)?;
        db::add_section(&conn, &params.get("title").cloned().unwrap_or_default())?;
        return Ok(ok_json(r#"{"ok":true}"#));
    }
    if let Some(id) = path.strip_prefix("/api/section/delete/") {
        let conn = db::get_conn(db_path)?;
        db::delete_section(&conn, id.parse()?)?; return Ok(ok_json(r#"{"ok":true}"#));
    }
    if let Some(rest) = path.strip_prefix("/api/section/move/") {
        let p: Vec<&str> = rest.split('/').collect();
        if p.len() == 2 { let conn = db::get_conn(db_path)?;
            db::move_section(&conn, p[0].parse()?, p[1])?; return Ok(ok_json(r#"{"ok":true}"#)); }
    }

    // ── API: pages ──────────────────────────────────────────────────────────
    if path == "/api/page/add" {
        let p = parse(body);
        let conn = db::get_conn(db_path)?;
        db::add_page(&conn, p.get("section_id").and_then(|s| s.parse().ok()).unwrap_or(0),
            &p.get("slug").cloned().unwrap_or_default(), &p.get("title").cloned().unwrap_or_default())?;
        return Ok(ok_json(r#"{"ok":true}"#));
    }
    if let Some(rest) = path.strip_prefix("/api/page/update/") {
        let id: i64 = rest.parse()?; let p = parse(body);
        let conn = db::get_conn(db_path)?;
        db::update_page(&conn, id, &p.get("slug").cloned().unwrap_or_default(),
            &p.get("title").cloned().unwrap_or_default(), &p.get("description").cloned().unwrap_or_default(),
            &p.get("keywords").cloned().unwrap_or_default(), &p.get("content").cloned().unwrap_or_default(),
            &p.get("seo_og_image").cloned().unwrap_or_default(), &p.get("seo_og_width").cloned().unwrap_or_default(),
            &p.get("seo_og_height").cloned().unwrap_or_default(), &p.get("seo_twitter_site").cloned().unwrap_or_default(),
            &p.get("seo_twitter_card").cloned().unwrap_or_default(), &p.get("seo_robots").cloned().unwrap_or_default())?;
        return Ok(ok_json(r#"{"ok":true}"#));
    }
    if let Some(id) = path.strip_prefix("/api/page/delete/") {
        let conn = db::get_conn(db_path)?; db::delete_page(&conn, id.parse()?)?;
        return Ok(ok_json(r#"{"ok":true}"#));
    }
    if let Some(rest) = path.strip_prefix("/api/page/move/") {
        let p: Vec<&str> = rest.split('/').collect();
        if p.len() == 2 { let conn = db::get_conn(db_path)?;
            db::move_page(&conn, p[0].parse()?, p[1])?; return Ok(ok_json(r#"{"ok":true}"#)); }
    }
    if let Some(id) = path.strip_prefix("/api/page/duplicate/") {
        let conn = db::get_conn(db_path)?; db::duplicate_page(&conn, id.parse()?)?;
        return Ok(ok_json(r#"{"ok":true}"#));
    }

    // ── API: settings ───────────────────────────────────────────────────────
    if path == "/api/settings" {
        let params = parse(body);
        let conn = db::get_conn(db_path)?;
        for (k, v) in &params { db::set_setting(&conn, k, v)?; }
        return Ok(ok_json(r#"{"ok":true}"#));
    }

    // ── Serve static files ──────────────────────────────────────────────────
    let file_url = if path == "/" { "/index.html" } else { path };
    let file_path = Path::new(build_dir).join(file_url.trim_start_matches('/'));

    if file_path.exists() && file_path.is_file() {
        let mut content = Vec::new();
        fs::File::open(&file_path)?.read_to_end(&mut content)?;
        let ct = mime(file_url);
        return Ok(Response::from_data(content).with_header(
            Header::from_bytes(b"Content-Type", format!("{}; charset=utf-8", ct).as_bytes()).unwrap()
        ));
    }

    // ── Fallback: redirect to admin ─────────────────────────────────────────
    if file_url == "/index.html" || path == "/" {
        return Ok(Response::from_string("").with_status_code(302)
            .with_header(Header::from_bytes(b"Location", b"/admin").unwrap()));
    }

    Ok(not_found())
}

fn build_resp(logs: &[String], ok: bool) -> Result<Response<std::io::Cursor<Vec<u8>>>> {
    let logs_json: Vec<String> = logs.iter().map(|l| format!(r#""{}""#, jesc(l))).collect();
    Ok(ok_json(&format!(r#"{{"ok":{},"logs":[{}]}}"#, ok, logs_json.join(","))))
}

fn parse(body: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for pair in body.split('&') {
        if let Some(eq) = pair.find('=') {
            map.insert(ud(&pair[..eq]), ud(&pair[eq+1..]));
        }
    }
    map
}

fn ud(s: &str) -> String {
    let mut r = String::new(); let mut c = s.chars();
    while let Some(ch) = c.next() {
        match ch {
            '+' => r.push(' '),
            '%' => { let h = c.next().and_then(|x| x.to_digit(16)).unwrap_or(0);
                     let l = c.next().and_then(|x| x.to_digit(16)).unwrap_or(0);
                     r.push(char::from((h*16+l) as u8)); }
            _ => r.push(ch),
        }
    }
    r
}

fn jesc(s: &str) -> String {
    let mut r = String::with_capacity(s.len()+2);
    for c in s.chars() {
        match c { '"' => r.push_str("\\\""), '\\' => r.push_str("\\\\"),
                   '\n' => r.push_str("\\n"), '\r' => r.push_str("\\r"), '\t' => r.push_str("\\t"), _ => r.push(c) }
    }
    r
}
