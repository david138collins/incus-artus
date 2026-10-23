//! Exporting — saving what's on screen as a PNG, for the art workflow.
//!
//! Pictures are saved one field pixel to one picture pixel (no smoothing, no scaling), exactly as
//! coloured on screen, in the `exports/` folder next to `assets/`.

use std::path::PathBuf;

pub fn export_folder() -> PathBuf {
    crate::debug::project_folder().join("exports")
}

/// Saves `width × height` colours (row by row, from the top left) as `exports/<name>.png`.
pub fn save_png(name: &str, width: usize, height: usize, colours: &[[u8; 3]]) -> Result<PathBuf, String> {
    let folder = export_folder();
    std::fs::create_dir_all(&folder).map_err(|error| error.to_string())?;
    let path = folder.join(format!("{name}.png"));
    let file = std::fs::File::create(&path).map_err(|error| error.to_string())?;
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
    writer.write_image_data(colours.as_flattened()).map_err(|error| error.to_string())?;
    writer.finish().map_err(|error| error.to_string())?;
    Ok(path)
}

/// A file name that won't clash with earlier exports: "noise-1761220000".
pub fn fresh_name(prefix: &str) -> String {
    let seconds = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let mut name = format!("{prefix}-{seconds}");
    let mut number = 2;
    while export_folder().join(format!("{name}.png")).exists() {
        name = format!("{prefix}-{seconds}-{number}");
        number += 1;
    }
    name
}

/// Opens a folder in Windows Explorer.
pub fn open_folder(folder: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(folder).map_err(|error| error.to_string())?;
    std::process::Command::new("explorer").arg(folder).spawn().map(|_| ()).map_err(|error| error.to_string())
}
