#![allow(dead_code)]
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::collections::HashMap;

#[derive(Clone)]
pub struct Section {
    pub id: i64,
    pub title: String,
    pub sort_order: i64,
}

#[derive(Clone)]
pub struct Page {
    pub id: i64,
    pub section_id: i64,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub keywords: String,
    pub content: String,
    pub sort_order: i64,
    pub seo_og_image: String,
    pub seo_og_width: String,
    pub seo_og_height: String,
    pub seo_twitter_site: String,
    pub seo_twitter_card: String,
    pub seo_robots: String,
}

pub fn get_conn(db_path: &str) -> Result<Connection> {
    let conn = Connection::open(db_path)
        .with_context(|| format!("Failed to open database: {}", db_path))?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    Ok(conn)
}

pub fn init_db(db_path: &str) -> Result<()> {
    let conn = get_conn(db_path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
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
        );",
    )?;

    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM settings",
        [],
        |row| row.get(0),
    )?;
    if count == 0 {
        let defaults = [
            ("SITE_NAME", "FreateOJ Wiki"),
            ("SITE_URL", "https://freatevietnam.github.io/freateoj-wiki"),
            ("SITE_DESCRIPTION", "Tài liệu và hướng dẫn về nền tảng chấm bài trực tuyến FreateOJ"),
            ("SITE_AUTHOR", "Freate Vietnam"),
            ("DEFAULT_LANG", "vi"),
            ("DEFAULT_LOCALE", "vi_VN"),
            ("OG_IMAGE", ""),
            ("OG_IMAGE_WIDTH", "1200"),
            ("OG_IMAGE_HEIGHT", "630"),
            ("TWITTER_SITE", "@freatevietnam"),
            ("TWITTER_CARD", "summary_large_image"),
            ("ROBOTS", "index, follow"),
            ("FAVICON_SVG", ""),
        ];
        for (k, v) in defaults {
            conn.execute(
                "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)",
                params![k, v],
            )?;
        }
    }
    Ok(())
}

pub fn get_all_settings(conn: &Connection) -> Result<HashMap<String, String>> {
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let rows = stmt.query_map([], |row| {
        let key: String = row.get(0)?;
        let value: String = row.get(1)?;
        Ok((key, value))
    })?;
    let mut map = HashMap::new();
    for row in rows {
        let (k, v) = row?;
        map.insert(k, v);
    }
    Ok(map)
}

pub fn get_setting(conn: &Connection, key: &str) -> Result<String> {
    let val = conn.query_row(
        "SELECT value FROM settings WHERE key=?1",
        params![key],
        |row| row.get::<_, String>(0),
    )?;
    Ok(val)
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    )?;
    Ok(())
}

pub fn get_all_sections(conn: &Connection) -> Result<Vec<Section>> {
    let mut stmt = conn.prepare("SELECT id, title, sort_order FROM sections ORDER BY sort_order")?;
    let rows = stmt.query_map([], |row| {
        Ok(Section {
            id: row.get(0)?,
            title: row.get(1)?,
            sort_order: row.get(2)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn get_all_pages(conn: &Connection) -> Result<Vec<Page>> {
    let mut stmt = conn.prepare(
        "SELECT id, section_id, slug, title, description, keywords, content, sort_order,
                seo_og_image, seo_og_width, seo_og_height, seo_twitter_site, seo_twitter_card, seo_robots
         FROM pages ORDER BY sort_order",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Page {
            id: row.get(0)?,
            section_id: row.get(1)?,
            slug: row.get(2)?,
            title: row.get(3)?,
            description: row.get(4)?,
            keywords: row.get(5)?,
            content: row.get(6)?,
            sort_order: row.get(7)?,
            seo_og_image: row.get(8)?,
            seo_og_width: row.get(9)?,
            seo_og_height: row.get(10)?,
            seo_twitter_site: row.get(11)?,
            seo_twitter_card: row.get(12)?,
            seo_robots: row.get(13)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn add_section(conn: &Connection, title: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO sections (title, sort_order) VALUES (?1, (SELECT COALESCE(MAX(sort_order),0)+1 FROM sections))",
        params![title],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn add_page(conn: &Connection, section_id: i64, slug: &str, title: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO pages (section_id, slug, title, sort_order) VALUES (?1, ?2, ?3, (SELECT COALESCE(MAX(sort_order),0)+1 FROM pages WHERE section_id=?1))",
        params![section_id, slug, title],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn delete_section(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM pages WHERE section_id=?1", params![id])?;
    conn.execute("DELETE FROM sections WHERE id=?1", params![id])?;
    Ok(())
}

pub fn delete_page(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM pages WHERE id=?1", params![id])?;
    Ok(())
}

pub fn update_page(
    conn: &Connection,
    id: i64,
    slug: &str,
    title: &str,
    description: &str,
    keywords: &str,
    content: &str,
    seo_og_image: &str,
    seo_og_width: &str,
    seo_og_height: &str,
    seo_twitter_site: &str,
    seo_twitter_card: &str,
    seo_robots: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE pages SET slug=?1, title=?2, description=?3, keywords=?4, content=?5,
         seo_og_image=?6, seo_og_width=?7, seo_og_height=?8, seo_twitter_site=?9, seo_twitter_card=?10, seo_robots=?11
         WHERE id=?12",
        params![
            slug, title, description, keywords, content,
            seo_og_image, seo_og_width, seo_og_height, seo_twitter_site, seo_twitter_card, seo_robots,
            id
        ],
    )?;
    Ok(())
}

pub fn move_page(conn: &Connection, id: i64, direction: &str) -> Result<()> {
    let page: Page = {
        let mut stmt = conn.prepare(
            "SELECT id, section_id, slug, title, description, keywords, content, sort_order,
                    seo_og_image, seo_og_width, seo_og_height, seo_twitter_site, seo_twitter_card, seo_robots
             FROM pages WHERE id=?1"
        )?;
        stmt.query_row(params![id], |row| {
            Ok(Page {
                id: row.get(0)?,
                section_id: row.get(1)?,
                slug: row.get(2)?,
                title: row.get(3)?,
                description: row.get(4)?,
                keywords: row.get(5)?,
                content: row.get(6)?,
                sort_order: row.get(7)?,
                seo_og_image: row.get(8)?,
                seo_og_width: row.get(9)?,
                seo_og_height: row.get(10)?,
                seo_twitter_site: row.get(11)?,
                seo_twitter_card: row.get(12)?,
                seo_robots: row.get(13)?,
            })
        })?
    };

    let other = if direction == "up" {
        conn.query_row(
            "SELECT id, sort_order FROM pages WHERE section_id=?1 AND sort_order<?2 ORDER BY sort_order DESC LIMIT 1",
            params![page.section_id, page.sort_order],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        )
    } else {
        conn.query_row(
            "SELECT id, sort_order FROM pages WHERE section_id=?1 AND sort_order>?2 ORDER BY sort_order ASC LIMIT 1",
            params![page.section_id, page.sort_order],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        )
    };

    if let Ok((other_id, other_order)) = other {
        conn.execute("UPDATE pages SET sort_order=?1 WHERE id=?2", params![other_order, id])?;
        conn.execute("UPDATE pages SET sort_order=?1 WHERE id=?2", params![page.sort_order, other_id])?;
    }
    Ok(())
}

pub fn move_section(conn: &Connection, id: i64, direction: &str) -> Result<()> {
    let cur_order: i64 = conn.query_row(
        "SELECT sort_order FROM sections WHERE id=?1",
        params![id],
        |row| row.get(0),
    )?;

    let other = if direction == "up" {
        conn.query_row(
            "SELECT id, sort_order FROM sections WHERE sort_order<?1 ORDER BY sort_order DESC LIMIT 1",
            params![cur_order],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        )
    } else {
        conn.query_row(
            "SELECT id, sort_order FROM sections WHERE sort_order>?1 ORDER BY sort_order ASC LIMIT 1",
            params![cur_order],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        )
    };

    if let Ok((other_id, other_order)) = other {
        conn.execute("UPDATE sections SET sort_order=?1 WHERE id=?2", params![other_order, id])?;
        conn.execute("UPDATE sections SET sort_order=?1 WHERE id=?2", params![cur_order, other_id])?;
    }
    Ok(())
}

pub fn duplicate_page(conn: &Connection, id: i64) -> Result<i64> {
    let page: Page = {
        let mut stmt = conn.prepare(
            "SELECT id, section_id, slug, title, description, keywords, content, sort_order,
                    seo_og_image, seo_og_width, seo_og_height, seo_twitter_site, seo_twitter_card, seo_robots
             FROM pages WHERE id=?1"
        )?;
        stmt.query_row(params![id], |row| {
            Ok(Page {
                id: row.get(0)?,
                section_id: row.get(1)?,
                slug: row.get(2)?,
                title: row.get(3)?,
                description: row.get(4)?,
                keywords: row.get(5)?,
                content: row.get(6)?,
                sort_order: row.get(7)?,
                seo_og_image: row.get(8)?,
                seo_og_width: row.get(9)?,
                seo_og_height: row.get(10)?,
                seo_twitter_site: row.get(11)?,
                seo_twitter_card: row.get(12)?,
                seo_robots: row.get(13)?,
            })
        })?
    };

    let mut new_slug = format!("{}-copy", page.slug);
    let mut i = 2;
    while conn.query_row(
        "SELECT 1 FROM pages WHERE slug=?1",
        params![new_slug],
        |_| Ok(()),
    ).is_ok() {
        new_slug = format!("{}-copy{}", page.slug, i);
        i += 1;
    }

    conn.execute(
        "INSERT INTO pages (section_id, slug, title, description, keywords, content, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, (SELECT COALESCE(MAX(sort_order),0)+1 FROM pages WHERE section_id=?1))",
        params![
            page.section_id, new_slug, format!("{} (copy)", page.title),
            page.description, page.keywords, page.content
        ],
    )?;
    let new_id = conn.last_insert_rowid();
    Ok(new_id)
}
