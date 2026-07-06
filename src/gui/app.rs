use anyhow::Result;
use eframe::egui::{self, Color32, CornerRadius, Stroke, Vec2, RichText, FontId, FontFamily, Margin};
use std::collections::HashMap;

use crate::build;
use crate::db::{self, Page, Section};

// ─── Theme System ────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ThemeMode { Dark, Light, Custom }

#[derive(Clone, Debug)]
pub struct Theme {
    pub name: String,
    #[allow(dead_code)]
    pub mode: ThemeMode,
    pub bg_primary: Color32,
    pub bg_secondary: Color32,
    pub bg_tertiary: Color32,
    pub bg_surface: Color32,
    pub bg_hover: Color32,
    pub bg_active: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub text_on_accent: Color32,
    pub accent: Color32,
    #[allow(dead_code)]
    pub accent_hover: Color32,
    pub accent_subtle: Color32,
    pub success: Color32,
    pub danger: Color32,
    pub border: Color32,
    #[allow(dead_code)]
    pub border_focus: Color32,
    pub sidebar_width: f32,
    pub corner_radius: u8,
    pub spacing: f32,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            name: "Midnight".into(), mode: ThemeMode::Dark,
            bg_primary: Color32::from_rgb(15, 15, 25),
            bg_secondary: Color32::from_rgb(22, 22, 35),
            bg_tertiary: Color32::from_rgb(30, 30, 45),
            bg_surface: Color32::from_rgb(26, 26, 40),
            bg_hover: Color32::from_rgb(38, 38, 58),
            bg_active: Color32::from_rgb(48, 48, 72),
            text_primary: Color32::from_rgb(235, 235, 245),
            text_secondary: Color32::from_rgb(170, 170, 190),
            text_muted: Color32::from_rgb(100, 100, 120),
            text_on_accent: Color32::from_rgb(255, 255, 255),
            accent: Color32::from_rgb(99, 102, 241),
            accent_hover: Color32::from_rgb(120, 120, 250),
            accent_subtle: Color32::from_rgb(30, 30, 60),
            success: Color32::from_rgb(34, 197, 94),
            danger: Color32::from_rgb(239, 68, 68),
            border: Color32::from_rgb(40, 40, 60),
            border_focus: Color32::from_rgb(99, 102, 241),
            sidebar_width: 250.0, corner_radius: 8, spacing: 6.0,
        }
    }

    pub fn light() -> Self {
        Self {
            name: "Daylight".into(), mode: ThemeMode::Light,
            bg_primary: Color32::from_rgb(248, 250, 252),
            bg_secondary: Color32::from_rgb(241, 245, 249),
            bg_tertiary: Color32::from_rgb(226, 232, 240),
            bg_surface: Color32::from_rgb(255, 255, 255),
            bg_hover: Color32::from_rgb(241, 245, 249),
            bg_active: Color32::from_rgb(226, 232, 240),
            text_primary: Color32::from_rgb(15, 23, 42),
            text_secondary: Color32::from_rgb(71, 85, 105),
            text_muted: Color32::from_rgb(148, 163, 184),
            text_on_accent: Color32::from_rgb(255, 255, 255),
            accent: Color32::from_rgb(99, 102, 241),
            accent_hover: Color32::from_rgb(80, 80, 220),
            accent_subtle: Color32::from_rgb(238, 240, 255),
            success: Color32::from_rgb(22, 163, 74),
            danger: Color32::from_rgb(220, 38, 38),
            border: Color32::from_rgb(226, 232, 240),
            border_focus: Color32::from_rgb(99, 102, 241),
            sidebar_width: 250.0, corner_radius: 8, spacing: 6.0,
        }
    }

    pub fn cyberpunk() -> Self {
        Self {
            name: "Cyberpunk".into(), mode: ThemeMode::Custom,
            bg_primary: Color32::from_rgb(10, 10, 15),
            bg_secondary: Color32::from_rgb(18, 18, 28),
            bg_tertiary: Color32::from_rgb(25, 25, 40),
            bg_surface: Color32::from_rgb(20, 20, 32),
            bg_hover: Color32::from_rgb(35, 35, 55),
            bg_active: Color32::from_rgb(45, 45, 70),
            text_primary: Color32::from_rgb(0, 255, 200),
            text_secondary: Color32::from_rgb(0, 200, 160),
            text_muted: Color32::from_rgb(0, 120, 100),
            text_on_accent: Color32::from_rgb(10, 10, 15),
            accent: Color32::from_rgb(0, 255, 200),
            accent_hover: Color32::from_rgb(0, 230, 180),
            accent_subtle: Color32::from_rgb(0, 40, 30),
            success: Color32::from_rgb(0, 255, 150),
            danger: Color32::from_rgb(255, 50, 80),
            border: Color32::from_rgb(0, 80, 60),
            border_focus: Color32::from_rgb(0, 255, 200),
            sidebar_width: 250.0, corner_radius: 4, spacing: 6.0,
        }
    }

    pub fn ocean() -> Self {
        Self {
            name: "Ocean".into(), mode: ThemeMode::Custom,
            bg_primary: Color32::from_rgb(13, 17, 38),
            bg_secondary: Color32::from_rgb(18, 24, 50),
            bg_tertiary: Color32::from_rgb(24, 32, 62),
            bg_surface: Color32::from_rgb(20, 28, 55),
            bg_hover: Color32::from_rgb(30, 40, 72),
            bg_active: Color32::from_rgb(38, 50, 85),
            text_primary: Color32::from_rgb(180, 210, 255),
            text_secondary: Color32::from_rgb(130, 160, 210),
            text_muted: Color32::from_rgb(80, 100, 150),
            text_on_accent: Color32::from_rgb(255, 255, 255),
            accent: Color32::from_rgb(56, 152, 236),
            accent_hover: Color32::from_rgb(70, 170, 255),
            accent_subtle: Color32::from_rgb(20, 40, 70),
            success: Color32::from_rgb(72, 199, 142),
            danger: Color32::from_rgb(239, 83, 80),
            border: Color32::from_rgb(30, 40, 70),
            border_focus: Color32::from_rgb(56, 152, 236),
            sidebar_width: 250.0, corner_radius: 10, spacing: 6.0,
        }
    }

    pub fn all() -> Vec<Self> { vec![Self::dark(), Self::light(), Self::cyberpunk(), Self::ocean()] }

    pub fn cr(&self) -> CornerRadius { CornerRadius::same(self.corner_radius) }
    pub fn cr_sm(&self) -> CornerRadius { CornerRadius::same((self.corner_radius as f32 * 0.6) as u8) }
}

// ─── Tabs ────────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug)]
enum Tab { Editor, Preview, Split, PageSeo, GlobalSeo, Stats, Build }

impl Tab {
    fn icon(&self) -> &str {
        match self { Tab::Editor => "✏", Tab::Preview => "👁", Tab::Split => "⬜", Tab::PageSeo => "⚙", Tab::GlobalSeo => "🌐", Tab::Stats => "📊", Tab::Build => "🔨" }
    }
    fn label(&self) -> &str {
        match self { Tab::Editor => "Editor", Tab::Preview => "Preview", Tab::Split => "Split", Tab::PageSeo => "Page SEO", Tab::GlobalSeo => "Global SEO", Tab::Stats => "Stats", Tab::Build => "Build" }
    }
}

// ─── Notification ────────────────────────────────────────────────────────────

#[derive(Clone)]
struct Notif { msg: String, kind: NotifKind, time: f32 }
#[derive(Clone, PartialEq)]
enum NotifKind { Success, Error, #[allow(dead_code)] Info }

// ─── App State ───────────────────────────────────────────────────────────────

pub struct WikiApp {
    db_path: String, output_dir: String,
    sections: Vec<Section>, pages: Vec<Page>, settings: HashMap<String, String>,
    selected_page_id: Option<i64>, selected_item: Option<(String, i64)>,
    edit_slug: String, edit_title: String, edit_desc: String, edit_keywords: String, edit_content: String,
    seo_og_image: String, seo_og_width: String, seo_og_height: String,
    seo_twitter_site: String, seo_twitter_card: String, seo_robots: String,
    global_seo: HashMap<String, String>, favicon_svg: String,
    current_tab: Tab, theme: Theme, show_theme_picker: bool,
    log_messages: Vec<String>, build_progress: f32, build_status: String, preview_html: String,
    show_add_section: bool, show_add_page: bool, show_delete_confirm: bool, show_settings: bool,
    new_section_name: String, new_page_section_idx: usize, new_page_title: String, new_page_slug: String,
    notifications: Vec<Notif>, sidebar_search: String, expanded: HashMap<i64, bool>,
}

impl WikiApp {
    pub fn new(db_path: &str, output_dir: &str) -> Result<Self> {
        db::init_db(db_path)?;
        let conn = db::get_conn(db_path)?;
        let settings = db::get_all_settings(&conn)?;
        let sections = db::get_all_sections(&conn)?;
        let pages = db::get_all_pages(&conn)?;
        let mut global_seo = HashMap::new();
        for key in ["SITE_NAME","SITE_URL","SITE_DESCRIPTION","SITE_AUTHOR","DEFAULT_LANG","DEFAULT_LOCALE","OG_IMAGE","OG_IMAGE_WIDTH","OG_IMAGE_HEIGHT","TWITTER_SITE","TWITTER_CARD","ROBOTS"] {
            global_seo.insert(key.to_string(), settings.get(key).cloned().unwrap_or_default());
        }
        let favicon_svg = settings.get("FAVICON_SVG").cloned().unwrap_or_default();
        let mut expanded = HashMap::new();
        for s in &sections { expanded.insert(s.id, true); }
        Ok(Self {
            db_path: db_path.into(), output_dir: output_dir.into(),
            sections, pages, settings, selected_page_id: None, selected_item: None,
            edit_slug: String::new(), edit_title: String::new(), edit_desc: String::new(),
            edit_keywords: String::new(), edit_content: String::new(),
            seo_og_image: String::new(), seo_og_width: String::new(), seo_og_height: String::new(),
            seo_twitter_site: String::new(), seo_twitter_card: String::new(), seo_robots: String::new(),
            global_seo, favicon_svg,             current_tab: Tab::Editor,
            theme: Theme::dark(), show_theme_picker: false,
            log_messages: Vec::new(), build_progress: 0.0, build_status: "Ready".into(),
            preview_html: String::new(),
            show_add_section: false, show_add_page: false, show_delete_confirm: false, show_settings: false,
            new_section_name: String::new(), new_page_section_idx: 0,
            new_page_title: String::new(), new_page_slug: String::new(),
            notifications: Vec::new(), sidebar_search: String::new(), expanded,
        })
    }

    fn refresh(&mut self) {
        if let Ok(c) = db::get_conn(&self.db_path) {
            if let Ok(s) = db::get_all_sections(&c) { self.sections = s; for s in &self.sections { self.expanded.entry(s.id).or_insert(true); } }
            if let Ok(p) = db::get_all_pages(&c) { self.pages = p; }
            if let Ok(s) = db::get_all_settings(&c) { self.settings = s; }
        }
    }

    fn load_page(&mut self, id: i64) {
        if let Some(p) = self.pages.iter().find(|p| p.id == id) {
            self.selected_page_id = Some(id);
            self.edit_slug = p.slug.clone(); self.edit_title = p.title.clone();
            self.edit_desc = p.description.clone(); self.edit_keywords = p.keywords.clone();
            self.edit_content = p.content.clone();
            self.seo_og_image = p.seo_og_image.clone(); self.seo_og_width = p.seo_og_width.clone();
            self.seo_og_height = p.seo_og_height.clone();
            self.seo_twitter_site = p.seo_twitter_site.clone(); self.seo_twitter_card = p.seo_twitter_card.clone();
            self.seo_robots = p.seo_robots.clone();
            self.preview_html = build::render_markdown(&p.content);
        }
    }

    fn save_page(&mut self) {
        if let Some(id) = self.selected_page_id {
            if let Ok(c) = db::get_conn(&self.db_path) {
                let _ = db::update_page(&c, id, &self.edit_slug, &self.edit_title, &self.edit_desc,
                    &self.edit_keywords, &self.edit_content, &self.seo_og_image, &self.seo_og_width,
                    &self.seo_og_height, &self.seo_twitter_site, &self.seo_twitter_card, &self.seo_robots);
                self.refresh(); self.notify("Page saved", NotifKind::Success);
            }
        }
    }

    fn run_build(&mut self) {
        self.log_messages.clear(); self.build_progress = 0.0; self.build_status = "Building...".into();
        if let Ok(c) = db::get_conn(&self.db_path) {
            let settings = db::get_all_settings(&c).unwrap_or_default();
            let sections = db::get_all_sections(&c).unwrap_or_default();
            let pages = db::get_all_pages(&c).unwrap_or_default();
            self.log_messages.push("Preparing build...".into()); self.build_progress = 0.2;
            match build::build(&settings, &sections, &pages, &self.output_dir) {
                Ok(_) => { self.build_progress = 1.0; self.build_status = "Done!".into(); self.notify("Build complete!", NotifKind::Success); }
                Err(e) => { self.build_status = format!("Failed: {}", e); self.notify(&format!("Error: {}", e), NotifKind::Error); }
            }
        }
    }

    fn notify(&mut self, msg: &str, kind: NotifKind) {
        self.notifications.push(Notif { msg: msg.into(), kind, time: 3.0 });
    }
}

// ─── eframe::App ─────────────────────────────────────────────────────────────

impl eframe::App for WikiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let t = self.theme.clone();

        // Apply style
        let mut style = (*ui.ctx().global_style()).clone();
        style.spacing.item_spacing = Vec2::new(t.spacing, t.spacing);
        style.spacing.button_padding = Vec2::new(12.0, 6.0);
        style.visuals.widgets.noninteractive.bg_fill = t.bg_tertiary;
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, t.text_secondary);
        style.visuals.widgets.inactive.bg_fill = t.bg_tertiary;
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, t.text_primary);
        style.visuals.widgets.inactive.corner_radius = t.cr();
        style.visuals.widgets.hovered.bg_fill = t.bg_hover;
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, t.text_primary);
        style.visuals.widgets.hovered.corner_radius = t.cr();
        style.visuals.widgets.active.bg_fill = t.bg_active;
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, t.accent);
        style.visuals.widgets.active.corner_radius = t.cr();
        style.visuals.selection.bg_fill = t.accent_subtle;
        style.visuals.selection.stroke = Stroke::new(1.0, t.accent);
        style.visuals.extreme_bg_color = t.bg_primary;
        style.visuals.faint_bg_color = t.bg_secondary;
        style.visuals.window_fill = t.bg_surface;
        style.visuals.window_stroke = Stroke::new(1.0, t.border);
        style.visuals.window_corner_radius = t.cr();
        ui.ctx().set_global_style(style);

        // Notifications
        self.notifications.retain(|n| n.time > 0.0);
        if !self.notifications.is_empty() {
            egui::Area::new(egui::Id::new("notifs")).anchor(egui::Align2::RIGHT_TOP, Vec2::new(-16.0, 16.0)).show(ui.ctx(), |ui| {
                for n in &mut self.notifications {
                    let bg = match n.kind { NotifKind::Success => t.success, NotifKind::Error => t.danger, NotifKind::Info => t.accent };
                    egui::Frame::new().fill(bg).corner_radius(t.cr()).inner_margin(Margin::symmetric(16, 10)).show(ui, |ui| {
                        ui.set_min_width(260.0);
                        ui.label(RichText::new(&n.msg).color(t.text_on_accent));
                    });
                    n.time -= ui.ctx().input(|i| i.predicted_dt);
                    ui.add_space(4.0);
                }
            });
        }

        // Layout
        egui::Panel::left("sidebar").exact_size(t.sidebar_width)
            .frame(egui::Frame::new().fill(t.bg_secondary).inner_margin(Margin::same(0)))
            .show_inside(ui, |ui| self.sidebar(ui));

        egui::Panel::top("topbar")
            .frame(egui::Frame::new().fill(t.bg_secondary).inner_margin(Margin::symmetric(12, 6)))
            .show_inside(ui, |ui| self.topbar(ui));

        egui::Panel::bottom("statusbar")
            .frame(egui::Frame::new().fill(t.bg_secondary).inner_margin(Margin::symmetric(12, 6)))
            .show_inside(ui, |ui| self.statusbar(ui));

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(t.bg_primary).inner_margin(Margin::same(16)))
            .show_inside(ui, |ui| self.main(ui));
    }
}

// ─── Sidebar ─────────────────────────────────────────────────────────────────

impl WikiApp {
    fn sidebar(&mut self, ui: &mut egui::Ui) {
        let t = self.theme.clone();

        // Logo
        egui::Frame::new().fill(t.bg_tertiary).inner_margin(Margin::symmetric(16, 14)).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("◆").color(t.accent).strong().size(20.0));
                ui.label(RichText::new("FreateOJ Wiki").color(t.text_primary).strong().size(15.0));
            });
            ui.add_space(4.0);
            ui.label(RichText::new(format!("{} pages · {} sections", self.pages.len(), self.sections.len())).color(t.text_muted).small());
        });

        // Search
        egui::Frame::new().inner_margin(Margin::symmetric(12, 8)).show(ui, |ui| {
            let mut s = self.sidebar_search.clone();
            let r = ui.add(egui::TextEdit::singleline(&mut s).hint_text(RichText::new("Search...").color(t.text_muted)).desired_width(f32::INFINITY).margin(Margin::symmetric(10, 8)));
            if r.changed() { self.sidebar_search = s; }
        });

        ui.separator();

        // Tree
        egui::ScrollArea::vertical().show(ui, |ui| {
            let search = self.sidebar_search.to_lowercase();
            for si in 0..self.sections.len() {
                let section_id = self.sections[si].id;
                let title = self.sections[si].title.clone();
                let is_exp = *self.expanded.get(&section_id).unwrap_or(&true);

                let pages: Vec<(i64, String)> = self.pages.iter()
                    .filter(|p| p.section_id == section_id)
                    .filter(|p| search.is_empty() || p.title.to_lowercase().contains(&search) || p.slug.to_lowercase().contains(&search))
                    .map(|p| (p.id, p.title.clone()))
                    .collect();

                egui::Frame::new().inner_margin(Margin::symmetric(12, 6)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let arrow = if is_exp { "▾" } else { "▸" };
                        let r = ui.add(egui::Label::new(RichText::new(format!("{} {}", arrow, title)).color(t.text_muted).strong().small()).sense(egui::Sense::click()));
                        if r.clicked() { let e = self.expanded.entry(section_id).or_insert(true); *e = !*e; }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(format!("{}", pages.len())).color(t.text_muted).small());
                        });
                    });
                });

                if is_exp {
                    for (pid, ptitle) in &pages {
                        let active = self.selected_page_id == Some(*pid);
                        let bg = if active { t.accent_subtle } else { Color32::TRANSPARENT };
                        let tc = if active { t.accent } else { t.text_secondary };
                        egui::Frame::new().fill(bg).corner_radius(t.cr_sm()).inner_margin(Margin::symmetric(12, 5)).show(ui, |ui| {
                            let r = ui.add(egui::Label::new(RichText::new(format!("  {}", ptitle)).color(tc).size(13.0)).sense(egui::Sense::click()));
                            if r.clicked() { self.selected_item = Some(("page".into(), *pid)); self.load_page(*pid); self.current_tab = Tab::Editor; }
                        });
                    }
                }
            }
        });

        ui.separator();

        // Actions
        egui::Frame::new().inner_margin(Margin::symmetric(12, 8)).show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new(RichText::new("+ Section").color(t.accent)).fill(Color32::TRANSPARENT)).clicked() { self.show_add_section = true; }
                if ui.add(egui::Button::new(RichText::new("+ Page").color(t.accent)).fill(Color32::TRANSPARENT)).clicked() && !self.sections.is_empty() { self.show_add_page = true; }
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let up = ui.add(egui::Button::new(RichText::new("↑").color(t.text_secondary)).fill(Color32::TRANSPARENT).min_size(Vec2::new(28.0, 24.0)));
                let dn = ui.add(egui::Button::new(RichText::new("↓").color(t.text_secondary)).fill(Color32::TRANSPARENT).min_size(Vec2::new(28.0, 24.0)));
                let dup = ui.add(egui::Button::new(RichText::new("⊕").color(t.text_secondary)).fill(Color32::TRANSPARENT).min_size(Vec2::new(28.0, 24.0)));
                let del = ui.add(egui::Button::new(RichText::new("🗑").color(t.danger)).fill(Color32::TRANSPARENT).min_size(Vec2::new(28.0, 24.0)));
                if up.clicked() || dn.clicked() || dup.clicked() || del.clicked() {
                    if let Some((ref it, id)) = self.selected_item.clone() {
                        if let Ok(c) = db::get_conn(&self.db_path) {
                            if up.clicked() { if it=="page" { let _=db::move_page(&c,id,"up"); } else { let _=db::move_section(&c,id,"up"); } }
                            else if dn.clicked() { if it=="page" { let _=db::move_page(&c,id,"down"); } else { let _=db::move_section(&c,id,"down"); } }
                            else if dup.clicked() && it=="page" { let _=db::duplicate_page(&c,id); }
                            else if del.clicked() { self.show_delete_confirm = true; }
                            self.refresh();
                        }
                    }
                }
            });
        });
    }

    // ─── Topbar ─────────────────────────────────────────────────────────────

    fn topbar(&mut self, ui: &mut egui::Ui) {
        let t = self.theme.clone();
        ui.horizontal(|ui| {
            for tab in [Tab::Editor, Tab::Preview, Tab::Split, Tab::PageSeo, Tab::GlobalSeo, Tab::Stats, Tab::Build] {
                let active = self.current_tab == tab;
                let (bg, tc) = if active { (t.accent_subtle, t.accent) } else { (Color32::TRANSPARENT, t.text_muted) };
                if ui.add(egui::Button::new(RichText::new(format!("{} {}", tab.icon(), tab.label())).color(tc).size(13.0)).fill(bg).corner_radius(t.cr_sm()).min_size(Vec2::new(0.0, 28.0))).clicked() {
                    self.current_tab = tab;
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(egui::Button::new(RichText::new(format!("🎨 {}", self.theme.name)).color(t.text_secondary).size(12.0)).fill(Color32::TRANSPARENT).corner_radius(t.cr_sm())).clicked() {
                    self.show_theme_picker = !self.show_theme_picker;
                }
                if ui.add(egui::Button::new(RichText::new("⚙").color(t.text_secondary).size(14.0)).fill(Color32::TRANSPARENT).corner_radius(t.cr_sm())).clicked() {
                    self.show_settings = !self.show_settings;
                }
            });
        });

        // Theme dropdown
        if self.show_theme_picker {
            egui::Area::new(egui::Id::new("theme_picker")).anchor(egui::Align2::RIGHT_TOP, Vec2::new(-80.0, 40.0)).show(ui.ctx(), |ui| {
                egui::Frame::new().fill(t.bg_surface).corner_radius(t.cr()).inner_margin(Margin::same(8)).show(ui, |ui| {
                    ui.set_min_width(160.0);
                    ui.label(RichText::new("Theme").color(t.text_muted).small().strong());
                    ui.add_space(4.0);
                    for theme in Theme::all() {
                        let sel = self.theme.name == theme.name;
                        let bg = if sel { t.accent_subtle } else { Color32::TRANSPARENT };
                        let tc = if sel { t.accent } else { t.text_primary };
                        if ui.add(egui::Button::new(RichText::new(&theme.name).color(tc)).fill(bg).corner_radius(t.cr_sm()).min_size(Vec2::new(140.0, 28.0))).clicked() {
                            self.theme = theme; self.show_theme_picker = false;
                        }
                    }
                });
            });
        }
    }

    // ─── Statusbar ──────────────────────────────────────────────────────────

    fn statusbar(&mut self, ui: &mut egui::Ui) {
        let t = self.theme.clone();
        ui.horizontal(|ui| {
            let sc = if self.build_status.contains("Done") { t.success } else if self.build_status.contains("Failed") { t.danger } else { t.text_muted };
            ui.label(RichText::new(&self.build_status).color(sc).small());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (label, color) = if self.build_status.contains("Server") { ("⏹ Stop", t.danger) } else { ("▶ Server", t.success) };
                if ui.add(egui::Button::new(RichText::new(label).color(t.text_on_accent).small()).fill(color).corner_radius(t.cr_sm()).min_size(Vec2::new(0.0, 24.0))).clicked() {}
            });
        });
    }

    // ─── Main ───────────────────────────────────────────────────────────────

    fn main(&mut self, ui: &mut egui::Ui) {
        match self.current_tab.clone() {
            Tab::Editor => self.tab_editor(ui),
            Tab::Preview => self.tab_preview(ui),
            Tab::Split => self.tab_split(ui),
            Tab::PageSeo => self.tab_page_seo(ui),
            Tab::GlobalSeo => self.tab_global_seo(ui),
            Tab::Stats => self.tab_stats(ui),
            Tab::Build => self.tab_build(ui),
        }
        self.dialogs(ui);
    }

    // ─── Editor ─────────────────────────────────────────────────────────────

    fn tab_editor(&mut self, ui: &mut egui::Ui) {
        let t = self.theme.clone();
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Frame::new().fill(t.bg_surface).corner_radius(t.cr()).inner_margin(Margin::same(20)).show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| { ui.label(RichText::new("✏").size(20.0)); ui.label(RichText::new("Page Editor").color(t.text_primary).strong().size(16.0)); });
                ui.add_space(8.0);

                egui::Grid::new("editor_meta").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
                    for (label, val) in [("Slug", &mut self.edit_slug), ("Title", &mut self.edit_title), ("Description", &mut self.edit_desc), ("Keywords", &mut self.edit_keywords)] {
                        ui.label(RichText::new(label).color(t.text_secondary));
                        ui.add(egui::TextEdit::singleline(val).desired_width(f32::INFINITY).margin(Margin::symmetric(8, 6)));
                        ui.end_row();
                    }
                });

                ui.add_space(12.0);
                ui.label(RichText::new("Content (Markdown)").color(t.text_secondary).strong());
                ui.add_space(4.0);
                egui::Frame::new().fill(t.bg_primary).corner_radius(t.cr()).inner_margin(Margin::same(4)).show(ui, |ui| {
                    ui.add(egui::TextEdit::multiline(&mut self.edit_content).desired_rows(25).desired_width(f32::INFINITY).font(FontId::new(13.0, FontFamily::Monospace)));
                });

                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new(RichText::new("👁 Preview").color(t.text_on_accent)).fill(t.accent).corner_radius(t.cr()).min_size(Vec2::new(0.0, 32.0))).clicked() {
                        self.preview_html = build::render_markdown(&self.edit_content); self.current_tab = Tab::Preview;
                    }
                    if ui.add(egui::Button::new(RichText::new("⬜ Split").color(t.text_on_accent)).fill(t.accent).corner_radius(t.cr()).min_size(Vec2::new(0.0, 32.0))).clicked() {
                        self.preview_html = build::render_markdown(&self.edit_content); self.current_tab = Tab::Split;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new(RichText::new("💾 Save").color(t.text_on_accent).strong()).fill(t.success).corner_radius(t.cr()).min_size(Vec2::new(100.0, 32.0))).clicked() { self.save_page(); }
                    });
                });
            });
        });
    }

    // ─── Preview ────────────────────────────────────────────────────────────

    fn tab_preview(&mut self, ui: &mut egui::Ui) {
        let t = self.theme.clone();
        egui::Frame::new().fill(t.bg_surface).corner_radius(t.cr()).inner_margin(Margin::same(20)).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new("👁").size(20.0));
                ui.label(RichText::new("HTML Preview").color(t.text_primary).strong().size(16.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new("🔄 Refresh").color(t.text_secondary)).fill(Color32::TRANSPARENT).corner_radius(t.cr_sm())).clicked() {
                        self.preview_html = build::render_markdown(&self.edit_content);
                    }
                });
            });
            ui.add_space(8.0);
            if self.preview_html.is_empty() {
                ui.centered_and_justified(|ui| { ui.label(RichText::new("No content to preview.").color(t.text_muted).size(14.0)); });
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Frame::new().fill(t.bg_primary).corner_radius(t.cr()).inner_margin(Margin::same(12)).show(ui, |ui| {
                        ui.add(egui::TextEdit::multiline(&mut self.preview_html.clone()).desired_rows(30).desired_width(f32::INFINITY).font(FontId::new(12.0, FontFamily::Monospace)));
                    });
                });
            }
        });
    }

    // ─── Split ──────────────────────────────────────────────────────────────

    fn tab_split(&mut self, ui: &mut egui::Ui) {
        let t = self.theme.clone();
        ui.columns(2, |cols| {
            egui::Frame::new().fill(t.bg_surface).corner_radius(t.cr()).inner_margin(Margin::same(12)).show(&mut cols[0], |ui| {
                ui.label(RichText::new("✏ Markdown").color(t.text_secondary).strong());
                ui.add_space(4.0);
                ui.add(egui::TextEdit::multiline(&mut self.edit_content).desired_rows(30).desired_width(f32::INFINITY).font(FontId::new(13.0, FontFamily::Monospace)));
                if ui.add(egui::Button::new(RichText::new("🔄 Update").color(t.text_on_accent)).fill(t.accent).corner_radius(t.cr())).clicked() {
                    self.preview_html = build::render_markdown(&self.edit_content);
                }
            });
            egui::Frame::new().fill(t.bg_surface).corner_radius(t.cr()).inner_margin(Margin::same(12)).show(&mut cols[1], |ui| {
                ui.label(RichText::new("👁 Preview").color(t.text_secondary).strong());
                ui.add_space(4.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if self.preview_html.is_empty() { ui.label(RichText::new("Click Update").color(t.text_muted)); }
                    else { ui.add(egui::TextEdit::multiline(&mut self.preview_html.clone()).desired_rows(28).desired_width(f32::INFINITY).font(FontId::new(12.0, FontFamily::Monospace))); }
                });
            });
        });
    }

    // ─── Page SEO ───────────────────────────────────────────────────────────

    fn tab_page_seo(&mut self, ui: &mut egui::Ui) {
        let t = self.theme.clone();
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Frame::new().fill(t.bg_surface).corner_radius(t.cr()).inner_margin(Margin::same(20)).show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| { ui.label(RichText::new("⚙").size(20.0)); ui.label(RichText::new("Page SEO").color(t.text_primary).strong().size(16.0)); });
                ui.add_space(12.0);
                egui::Grid::new("page_seo").num_columns(2).spacing([12.0, 10.0]).show(ui, |ui| {
                    for (label, val) in [("OG Image URL", &mut self.seo_og_image), ("OG Width", &mut self.seo_og_width), ("OG Height", &mut self.seo_og_height), ("Twitter Site", &mut self.seo_twitter_site), ("Twitter Card", &mut self.seo_twitter_card), ("Robots", &mut self.seo_robots)] {
                        ui.label(RichText::new(label).color(t.text_secondary));
                        ui.add(egui::TextEdit::singleline(val).desired_width(f32::INFINITY).margin(Margin::symmetric(8, 6)));
                        ui.end_row();
                    }
                });
                ui.add_space(16.0);
                if ui.add(egui::Button::new(RichText::new("💾 Save SEO").color(t.text_on_accent)).fill(t.accent).corner_radius(t.cr()).min_size(Vec2::new(0.0, 32.0))).clicked() { self.save_page(); }
            });
        });
    }

    // ─── Global SEO ─────────────────────────────────────────────────────────

    fn tab_global_seo(&mut self, ui: &mut egui::Ui) {
        let t = self.theme.clone();
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Frame::new().fill(t.bg_surface).corner_radius(t.cr()).inner_margin(Margin::same(20)).show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| { ui.label(RichText::new("🌐").size(20.0)); ui.label(RichText::new("Global SEO").color(t.text_primary).strong().size(16.0)); });
                ui.add_space(12.0);
                let keys = [("SITE_NAME","Name"),("SITE_URL","URL"),("SITE_DESCRIPTION","Description"),("SITE_AUTHOR","Author"),("DEFAULT_LANG","Lang"),("DEFAULT_LOCALE","Locale"),("OG_IMAGE","OG Image"),("OG_IMAGE_WIDTH","OG W"),("OG_IMAGE_HEIGHT","OG H"),("TWITTER_SITE","Twitter"),("TWITTER_CARD","Card"),("ROBOTS","Robots")];
                egui::Grid::new("global_seo").num_columns(2).spacing([12.0, 10.0]).show(ui, |ui| {
                    for (key, label) in keys {
                        ui.label(RichText::new(label).color(t.text_secondary));
                        let v = self.global_seo.entry(key.to_string()).or_default();
                        ui.add(egui::TextEdit::singleline(v).desired_width(f32::INFINITY).margin(Margin::symmetric(8, 6)));
                        ui.end_row();
                    }
                });
                ui.add_space(12.0);
                ui.label(RichText::new("Favicon SVG").color(t.text_secondary));
                ui.add(egui::TextEdit::singleline(&mut self.favicon_svg).desired_width(f32::INFINITY).margin(Margin::symmetric(8, 6)));
                ui.add_space(16.0);
                if ui.add(egui::Button::new(RichText::new("💾 Save Global SEO").color(t.text_on_accent)).fill(t.accent).corner_radius(t.cr()).min_size(Vec2::new(0.0, 32.0))).clicked() {
                    if let Ok(c) = db::get_conn(&self.db_path) {
                        for (k, v) in &self.global_seo { let _ = db::set_setting(&c, k, v); }
                        let _ = db::set_setting(&c, "FAVICON_SVG", &self.favicon_svg);
                        self.refresh(); self.notify("Saved", NotifKind::Success);
                    }
                }
            });
        });
    }

    // ─── Stats ──────────────────────────────────────────────────────────────

    fn tab_stats(&mut self, ui: &mut egui::Ui) {
        let t = self.theme.clone();
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Frame::new().fill(t.bg_surface).corner_radius(t.cr()).inner_margin(Margin::same(20)).show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(RichText::new("📊").size(20.0)); ui.label(RichText::new("Statistics").color(t.text_primary).strong().size(16.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new(RichText::new("🔄 Refresh").color(t.text_secondary)).fill(Color32::TRANSPARENT).corner_radius(t.cr_sm())).clicked() { self.refresh(); }
                    });
                });
                ui.add_space(16.0);

                let tp = self.pages.len(); let ts = self.sections.len();
                let tw: usize = self.pages.iter().map(|p| p.content.split_whitespace().count()).sum();
                let tc: usize = self.pages.iter().map(|p| p.content.len()).sum();
                let aw = if tp > 0 { tw / tp } else { 0 };
                let rt = std::cmp::max(1, tw / 200);

                egui::Grid::new("stat_cards").num_columns(3).spacing([12.0, 12.0]).show(ui, |ui| {
                    for (icon, label, val) in [("📄","Pages",tp.to_string()),("📁","Sections",ts.to_string()),("📝","Words",format!("{}",tw)),("🔤","Chars",format!("{}",tc)),("📊","Avg/Page",aw.to_string()),("⏱","Read Time",format!("{} min",rt))] {
                        egui::Frame::new().fill(t.bg_primary).corner_radius(t.cr()).inner_margin(Margin::symmetric(16, 12)).show(ui, |ui| {
                            ui.set_min_width(150.0);
                            ui.label(RichText::new(icon).size(20.0)); ui.add_space(4.0);
                            ui.label(RichText::new(label).color(t.text_muted).small());
                            ui.label(RichText::new(val).color(t.text_primary).strong().size(18.0));
                        });
                    }
                });
            });
        });
    }

    // ─── Build ──────────────────────────────────────────────────────────────

    fn tab_build(&mut self, ui: &mut egui::Ui) {
        let t = self.theme.clone();
        egui::Frame::new().fill(t.bg_surface).corner_radius(t.cr()).inner_margin(Margin::same(20)).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| { ui.label(RichText::new("🔨").size(20.0)); ui.label(RichText::new("Build").color(t.text_primary).strong().size(16.0)); });
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new(RichText::new("🔨 Build Site").color(t.text_on_accent).strong()).fill(t.accent).corner_radius(t.cr()).min_size(Vec2::new(0.0, 36.0))).clicked() { self.run_build(); }
                if ui.add(egui::Button::new(RichText::new("📂 Open Folder").color(t.text_secondary)).fill(Color32::TRANSPARENT).corner_radius(t.cr_sm())).clicked() {
                    #[cfg(target_os = "windows")] { let _ = std::process::Command::new("explorer").arg(&self.output_dir).spawn(); }
                    #[cfg(target_os = "macos")] { let _ = std::process::Command::new("open").arg(&self.output_dir).spawn(); }
                    #[cfg(target_os = "linux")] { let _ = std::process::Command::new("xdg-open").arg(&self.output_dir).spawn(); }
                }
            });
            ui.add_space(12.0);
            let pc = if self.build_progress >= 1.0 { t.success } else { t.accent };
            ui.add(egui::ProgressBar::new(self.build_progress).fill(pc).text(RichText::new(&self.build_status).color(t.text_primary)));
            ui.add_space(12.0);
            ui.label(RichText::new("Build Log").color(t.text_secondary).strong());
            ui.add_space(4.0);
            egui::Frame::new().fill(t.bg_primary).corner_radius(t.cr()).inner_margin(Margin::same(8)).show(ui, |ui| {
                egui::ScrollArea::vertical().max_height(250.0).stick_to_bottom(true).show(ui, |ui| {
                    if self.log_messages.is_empty() { ui.label(RichText::new("No logs.").color(t.text_muted)); }
                    else { for m in &self.log_messages { ui.label(RichText::new(m).color(t.text_secondary).small()); } }
                });
            });
        });
    }

    // ─── Dialogs ────────────────────────────────────────────────────────────

    fn dialogs(&mut self, ui: &mut egui::Ui) {
        let t = self.theme.clone();

        if self.show_add_section {
            egui::Window::new("Add Section").collapsible(false).resizable(false)
                .frame(egui::Frame::new().fill(t.bg_surface).corner_radius(t.cr()).inner_margin(Margin::same(20)))
                .show(ui.ctx(), |ui| {
                    ui.label(RichText::new("Section Name").color(t.text_secondary));
                    ui.add(egui::TextEdit::singleline(&mut self.new_section_name).desired_width(300.0).margin(Margin::symmetric(8, 6)));
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new(RichText::new("Cancel").color(t.text_secondary)).fill(Color32::TRANSPARENT).corner_radius(t.cr_sm())).clicked() { self.show_add_section = false; self.new_section_name.clear(); }
                        if ui.add(egui::Button::new(RichText::new("Add").color(t.text_on_accent)).fill(t.accent).corner_radius(t.cr())).clicked() {
                            if !self.new_section_name.is_empty() {
                                if let Ok(c) = db::get_conn(&self.db_path) { let _ = db::add_section(&c, &self.new_section_name); self.refresh(); self.notify("Section added", NotifKind::Success); }
                                self.show_add_section = false; self.new_section_name.clear();
                            }
                        }
                    });
                });
        }

        if self.show_add_page {
            egui::Window::new("Add Page").collapsible(false).resizable(false)
                .frame(egui::Frame::new().fill(t.bg_surface).corner_radius(t.cr()).inner_margin(Margin::same(20)))
                .show(ui.ctx(), |ui| {
                    ui.label(RichText::new("Section").color(t.text_secondary));
                    egui::ComboBox::from_id_salt("section_combo").selected_text(self.sections.get(self.new_page_section_idx).map(|s| s.title.as_str()).unwrap_or("None")).show_ui(ui, |ui| {
                        for (i, s) in self.sections.iter().enumerate() { ui.selectable_value(&mut self.new_page_section_idx, i, &s.title); }
                    });
                    ui.add_space(8.0);
                    ui.label(RichText::new("Title").color(t.text_secondary));
                    ui.add(egui::TextEdit::singleline(&mut self.new_page_title).desired_width(300.0).margin(Margin::symmetric(8, 6)));
                    ui.add_space(4.0);
                    ui.label(RichText::new("Slug").color(t.text_secondary));
                    ui.add(egui::TextEdit::singleline(&mut self.new_page_slug).desired_width(300.0).margin(Margin::symmetric(8, 6)));
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new(RichText::new("Cancel").color(t.text_secondary)).fill(Color32::TRANSPARENT).corner_radius(t.cr_sm())).clicked() { self.show_add_page = false; self.new_page_title.clear(); self.new_page_slug.clear(); }
                        if ui.add(egui::Button::new(RichText::new("Add").color(t.text_on_accent)).fill(t.accent).corner_radius(t.cr())).clicked() {
                            if !self.new_page_title.is_empty() && !self.new_page_slug.is_empty() {
                                if let Some(s) = self.sections.get(self.new_page_section_idx) {
                                    if let Ok(c) = db::get_conn(&self.db_path) { let _ = db::add_page(&c, s.id, &self.new_page_slug, &self.new_page_title); self.refresh(); self.notify("Page added", NotifKind::Success); }
                                }
                                self.show_add_page = false; self.new_page_title.clear(); self.new_page_slug.clear();
                            }
                        }
                    });
                });
        }

        if self.show_delete_confirm {
            let name = match &self.selected_item {
                Some((t, id)) if t == "section" => self.sections.iter().find(|s| s.id == *id).map(|s| s.title.clone()),
                Some((t, id)) if t == "page" => self.pages.iter().find(|p| p.id == *id).map(|p| p.title.clone()),
                _ => None,
            }.unwrap_or_else(|| "Unknown".into());

            egui::Window::new("Delete").collapsible(false).resizable(false)
                .frame(egui::Frame::new().fill(t.bg_surface).corner_radius(t.cr()).inner_margin(Margin::same(20)))
                .show(ui.ctx(), |ui| {
                    ui.label(RichText::new(format!("Delete '{}'?", name)).color(t.text_primary));
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new(RichText::new("Cancel").color(t.text_secondary)).fill(Color32::TRANSPARENT).corner_radius(t.cr_sm())).clicked() { self.show_delete_confirm = false; }
                        if ui.add(egui::Button::new(RichText::new("Delete").color(t.text_on_accent)).fill(t.danger).corner_radius(t.cr())).clicked() {
                            if let Some((ref it, id)) = self.selected_item.clone() {
                                if let Ok(c) = db::get_conn(&self.db_path) {
                                    if it == "section" { let _ = db::delete_section(&c, id); } else { let _ = db::delete_page(&c, id); }
                                    self.refresh(); self.selected_item = None; self.selected_page_id = None; self.notify("Deleted", NotifKind::Success);
                                }
                            }
                            self.show_delete_confirm = false;
                        }
                    });
                });
        }

        if self.show_settings {
            egui::Window::new("Settings").collapsible(false).resizable(false)
                .frame(egui::Frame::new().fill(t.bg_surface).corner_radius(t.cr()).inner_margin(Margin::same(20)))
                .show(ui.ctx(), |ui| {
                    ui.label(RichText::new("App Settings").color(t.text_primary).strong().size(14.0));
                    ui.add_space(12.0);
                    ui.label(RichText::new("Database").color(t.text_secondary));
                    ui.label(RichText::new(&self.db_path).color(t.text_primary).monospace());
                    ui.add_space(8.0);
                    ui.label(RichText::new("Output").color(t.text_secondary));
                    ui.label(RichText::new(&self.output_dir).color(t.text_primary).monospace());
                    ui.add_space(12.0);
                    if ui.add(egui::Button::new(RichText::new("Close").color(t.text_secondary)).fill(Color32::TRANSPARENT).corner_radius(t.cr_sm())).clicked() { self.show_settings = false; }
                });
        }
    }
}
