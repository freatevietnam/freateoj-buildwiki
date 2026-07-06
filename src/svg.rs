use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

const ICO_SIZES: &[u32] = &[16, 32, 48, 64, 128, 256];

pub fn svg_to_ico(svg_path: &str, ico_path: &Path) -> Result<()> {
    let svg_data = fs::read(svg_path)
        .with_context(|| format!("Failed to read SVG file: {}", svg_path))?;

    let mut fontdb = resvg::usvg::fontdb::Database::new();
    fontdb.load_system_fonts();

    let opts = resvg::usvg::Options {
        resources_dir: Some(
            std::path::PathBuf::from(svg_path)
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf(),
        ),
        fontdb: std::sync::Arc::new(fontdb),
        ..Default::default()
    };

    let tree = resvg::usvg::Tree::from_data(&svg_data, &opts)
        .context("Failed to parse SVG")?;

    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

    for &size in ICO_SIZES {
        let pixmap_size = size * 2;
        let mut pixmap = tiny_skia::Pixmap::new(pixmap_size, pixmap_size)
            .context("Failed to create pixmap")?;

        let transform = tiny_skia::Transform::from_scale(
            pixmap_size as f32 / tree.size().width() as f32,
            pixmap_size as f32 / tree.size().height() as f32,
        );
        let mut pixmap_ref = pixmap.as_mut();
        resvg::render(&tree, transform, &mut pixmap_ref);

        let mut target_pixmap = tiny_skia::Pixmap::new(size, size)
            .context("Failed to create target pixmap")?;

        let scale_x = pixmap_size as f32 / size as f32;
        let scale_y = pixmap_size as f32 / size as f32;

        let target_pixels = target_pixmap.pixels_mut();
        for y in 0..size {
            for x in 0..size {
                let src_x = (x as f32 * scale_x) as u32;
                let src_y = (y as f32 * scale_y) as u32;
                if let Some(pixel) = pixmap.pixel(src_x, src_y) {
                    let idx = (y * size + x) as usize;
                    target_pixels[idx] = pixel;
                }
            }
        }

        let rgba = target_pixmap.data();
        let icon_image = ico::IconImage::from_rgba_data(size, size, rgba.to_vec());

        let entry = ico::IconDirEntry::encode_as_bmp(&icon_image)
            .context("Failed to encode ICO entry")?;
        icon_dir.add_entry(entry);
    }

    let mut ico_file = fs::File::create(ico_path)
        .with_context(|| format!("Failed to create ICO file: {}", ico_path.display()))?;

    icon_dir
        .write(&mut ico_file)
        .context("Failed to write ICO file")?;

    Ok(())
}
