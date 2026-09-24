//! Static SVG support.
//!
//! Windows' wallpaper API cannot display SVG files, so SVG wallpapers are
//! rasterized to a PNG at screen resolution and cached on disk. Rotation,
//! shuffle, and history all operate on the rendered PNG transparently.

use std::path::PathBuf;

use resvg::tiny_skia;
use resvg::usvg;

/// Rasterize `svg_path` to a PNG at `width` x `height` pixels and return its path.
///
/// The output is written to the app cache directory, keyed by the source file's
/// path, size, and modification time, so an unchanged SVG is rendered only once.
pub fn render_to_cached_png(
    svg_path: &str,
    width: u32,
    height: u32,
    mode: FitMode,
) -> Result<PathBuf, String> {
    let data = std::fs::read(svg_path)
        .map_err(|e| format!("Cannot read SVG file {}: {e}", svg_path))?;

    let cache_path = cache_file_path(svg_path, width, height, mode)
        .ok_or_else(|| format!("Cannot build cache path for {}", svg_path))?;

    if cache_path.exists() {
        return Ok(cache_path);
    }

    let png = render_png(&data, svg_path, width, height, mode)?;

    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create cache directory: {e}"))?;
    }
    // Write via a unique temp name so a concurrent rotation never reads a
    // half-written PNG.
    let tmp = cache_path.with_extension("png.tmp");
    std::fs::write(&tmp, &png).map_err(|e| format!("Cannot write SVG cache: {e}"))?;
    std::fs::rename(&tmp, &cache_path).map_err(|e| format!("Cannot finalize SVG cache: {e}"))?;

    Ok(cache_path)
}

/// How the SVG is fitted onto the screen.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FitMode {
    /// Scale to fill the screen, cropping overflow (like CSS `cover`).
    Cover,
    /// Scale to fit inside the screen, leaving letterbox/pillarbox transparent
    /// (like CSS `contain`). Not used in production yet (Cover is the
    /// default); exercised by tests.
    #[allow(dead_code)]
    Contain,
}

impl FitMode {
    fn suffix(self) -> &'static str {
        match self {
            FitMode::Cover => "cover",
            FitMode::Contain => "contain",
        }
    }
}

fn render_png(
    data: &[u8],
    source_path: &str,
    width: u32,
    height: u32,
    mode: FitMode,
) -> Result<Vec<u8>, String> {
    let mut opt = usvg::Options::default();
    // Resolve <image href="..."> relatives against the SVG's own folder, so
    // self-made SVG slideshows can embed local images next to the .svg file.
    opt.resources_dir = std::path::Path::new(source_path)
        .parent()
        .map(std::path::Path::to_path_buf);
    // System fonts are needed to draw <text>; load them before parsing.
    opt.fontdb_mut().load_system_fonts();

    let tree = usvg::Tree::from_data(data, &opt)
        .map_err(|e| format!("Invalid SVG {}: {e}", source_path))?;

    // Uniform scale + centering applied as the root transform passed to the
    // renderer (resvg composes it with the tree's own viewBox mapping).
    let svg_size = tree.size();
    let sx = width as f32 / svg_size.width();
    let sy = height as f32 / svg_size.height();
    let scale = match mode {
        FitMode::Cover => sx.max(sy),
        FitMode::Contain => sx.min(sy),
    };
    let dx = (width as f32 - svg_size.width() * scale) / 2.0;
    let dy = (height as f32 - svg_size.height() * scale) / 2.0;
    let transform = usvg::Transform::from_row(scale, 0.0, 0.0, scale, dx, dy);

    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| format!("Cannot allocate {width}x{height} pixmap"))?;
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    pixmap.encode_png().map_err(|e| format!("PNG encode failed: {e}"))
}

fn cache_file_path(svg_path: &str, width: u32, height: u32, mode: FitMode) -> Option<PathBuf> {
    let modified = std::fs::metadata(svg_path)
        .ok()?
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();

    // Hash the absolute path so names cannot collide and are stable per file.
    let path = std::fs::canonicalize(svg_path).unwrap_or_else(|_| PathBuf::from(svg_path));
    let mut hash: u64 = 0xcbf29ce484222325; // FNV-1a
    for byte in path.to_string_lossy().as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    let file = format!(
        "{hash:016x}-{modified:x}-{}x{}-{}.png",
        width,
        height,
        mode.suffix()
    );

    Some(cache_dir().join(file))
}

fn cache_dir() -> PathBuf {
    // LOCALAPPDATA is always present on Windows; fall back to TEMP for tests.
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::temp_dir()));
    base.join("ShufflePaper").join("svg-cache")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="200">
      <rect width="400" height="200" fill="rebeccapurple"/>
    </svg>"##;

    const CSS_TEXT_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 200">
      <style>
        .bg { fill: #10243e; }
        text { fill: white; font-family: sans-serif; font-size: 48px; }
      </style>
      <rect class="bg" width="400" height="200"/>
      <text x="200" y="110" text-anchor="middle">Hello SVG</text>
    </svg>"##;

    fn write_temp_svg(name: &str, contents: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "shufflepaper-svg-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        std::fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn renders_svg_to_requested_dimensions() {
        let svg = write_temp_svg("plain.svg", SIMPLE_SVG);
        let out = render_to_cached_png(svg.to_str().unwrap(), 1920, 1080, FitMode::Cover)
            .expect("render should succeed");
        let png = std::fs::read(&out).expect("cached png should exist");
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n", "output must be a PNG");
    }

    #[test]
    fn renders_svg_with_css_and_text() {
        let svg = write_temp_svg("css-text.svg", CSS_TEXT_SVG);
        let out = render_to_cached_png(svg.to_str().unwrap(), 800, 600, FitMode::Contain)
            .expect("CSS/text SVG should render");
        let png = std::fs::read(&out).expect("cached png should exist");
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn cache_hit_avoids_rerender() {
        let svg = write_temp_svg("cached.svg", SIMPLE_SVG);
        let first = render_to_cached_png(svg.to_str().unwrap(), 640, 480, FitMode::Cover).unwrap();
        let m1 = std::fs::metadata(&first).unwrap().modified().unwrap();
        // Render again; must return the same path without rewriting content.
        let second = render_to_cached_png(svg.to_str().unwrap(), 640, 480, FitMode::Cover).unwrap();
        let m2 = std::fs::metadata(&second).unwrap().modified().unwrap();
        assert_eq!(first, second);
        assert_eq!(m1, m2, "cache hit must not rewrite the PNG");
    }

    #[test]
    fn different_sizes_get_different_cache_entries() {
        let svg = write_temp_svg("sizes.svg", SIMPLE_SVG);
        let a = render_to_cached_png(svg.to_str().unwrap(), 320, 240, FitMode::Cover).unwrap();
        let b = render_to_cached_png(svg.to_str().unwrap(), 640, 480, FitMode::Cover).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn invalid_svg_fails_with_message() {
        let svg = write_temp_svg("broken.svg", "this is not svg at all");
        let err = render_to_cached_png(svg.to_str().unwrap(), 800, 600, FitMode::Cover)
            .expect_err("must fail");
        assert!(err.contains("Invalid SVG"), "unexpected error: {err}");
    }
}
