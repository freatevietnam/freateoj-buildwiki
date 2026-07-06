mod app;

use anyhow::Result;
use eframe::egui;
use std::sync::Arc;

/// Try to load a system font that supports Vietnamese/UTF-8.
fn try_load_system_font() -> Option<Vec<u8>> {
    let font_paths: Vec<&str> = if cfg!(windows) {
        vec![
            "C:\\Windows\\Fonts\\segoeui.ttf",
            "C:\\Windows\\Fonts\\arial.ttf",
            "C:\\Windows\\Fonts\\tahoma.ttf",
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/Supplemental/Arial.ttf",
            "/Library/Fonts/Arial Unicode.ttf",
        ]
    } else {
        vec![
            "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/truetype/freefont/FreeSans.ttf",
        ]
    };

    for path in font_paths {
        if let Ok(data) = std::fs::read(path) {
            return Some(data);
        }
    }
    None
}

/// Load system font for Vietnamese/UTF-8 support into egui.
fn load_utf8_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    if let Some(font_data) = try_load_system_font() {
        fonts.font_data.insert(
            "SystemFont".to_owned(),
            Arc::new(egui::FontData::from_owned(font_data)),
        );
        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap()
            .insert(0, "SystemFont".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap()
            .insert(0, "SystemFont".to_owned());
    }

    ctx.set_fonts(fonts);
}

pub fn run_gui(db_path: &str, output_dir: &str) -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 850.0])
            .with_min_inner_size([1000.0, 650.0])
            .with_title("FreateOJ Wiki Builder"),
        ..Default::default()
    };

    let app = app::WikiApp::new(db_path, output_dir)?;

    eframe::run_native(
        "FreateOJ Wiki Builder",
        options,
        Box::new(move |cc| {
            load_utf8_fonts(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run GUI: {}", e))?;

    Ok(())
}
