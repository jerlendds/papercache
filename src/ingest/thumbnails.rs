use std::{path::Path, process::Command};

use image::{GenericImageView, ImageBuffer, ImageFormat, Rgb};
use pdfium_render::prelude::*;

pub struct CoverImage {
    pub width: i64,
    pub height: i64,
    pub version: &'static str,
}

pub fn write_pdf_cover(pdf_path: &Path, cover_path: &Path) -> anyhow::Result<CoverImage> {
    if let Some(parent) = cover_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    render_with_pdfium(pdf_path, cover_path).or_else(|pdfium_error| {
        render_with_pdftoppm(pdf_path, cover_path).map_err(|poppler_error| {
            anyhow::anyhow!(
                "pdfium render failed: {pdfium_error}; pdftoppm render failed: {poppler_error}"
            )
        })
    })
}

fn render_with_pdfium(pdf_path: &Path, cover_path: &Path) -> anyhow::Result<CoverImage> {
    let bindings = Pdfium::bind_to_system_library()
        .or_else(|_| Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(".")))?;
    let pdfium = Pdfium::new(bindings);
    let document = pdfium.load_pdf_from_file(pdf_path, None)?;
    let page = document.pages().first()?;
    let image = page
        .render_with_config(
            &PdfRenderConfig::new()
                .set_target_width(640)
                .set_maximum_height(900)
                .render_form_data(true),
        )?
        .as_image()?
        .into_rgb8();
    let (width, height) = image.dimensions();
    image.save_with_format(cover_path, ImageFormat::Jpeg)?;

    Ok(CoverImage {
        width: i64::from(width),
        height: i64::from(height),
        version: "pdfium-jpeg-v1",
    })
}

fn render_with_pdftoppm(pdf_path: &Path, cover_path: &Path) -> anyhow::Result<CoverImage> {
    let temp_prefix = cover_path.with_file_name(format!(
        "{}.tmp-cover",
        cover_path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("cover")
    ));
    let temp_jpeg = temp_prefix.with_extension("tmp-cover.jpg");
    let _ = std::fs::remove_file(&temp_jpeg);

    let output = Command::new("pdftoppm")
        .arg("-jpeg")
        .arg("-singlefile")
        .arg("-f")
        .arg("1")
        .arg("-l")
        .arg("1")
        .arg("-scale-to-x")
        .arg("640")
        .arg("-scale-to-y")
        .arg("-1")
        .arg(pdf_path)
        .arg(&temp_prefix)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "pdftoppm exited with {}; stderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let image = image::open(&temp_jpeg)?;
    let (width, height) = image.dimensions();
    std::fs::rename(&temp_jpeg, cover_path)?;

    Ok(CoverImage {
        width: i64::from(width),
        height: i64::from(height),
        version: "poppler-jpeg-v1",
    })
}

pub fn write_placeholder_cover(path: &Path) -> anyhow::Result<CoverImage> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let width = 640;
    let height = 900;
    let image = ImageBuffer::from_fn(width, height, |x, y| {
        let border = x < 10 || x >= width - 10 || y < 10 || y >= height - 10;
        let page = x >= 56 && x < width - 56 && y >= 56 && y < height - 56;
        let line = page && y % 34 < 2 && x >= 100 && x < width - 100;
        let dot = page && x % 26 < 2 && y % 26 < 2;

        if border {
            Rgb([23_u8, 34, 60])
        } else if line {
            Rgb([202_u8, 217, 210])
        } else if dot {
            Rgb([210_u8, 200, 170])
        } else if page {
            Rgb([255_u8, 251, 233])
        } else {
            Rgb([247_u8, 241, 219])
        }
    });
    image.save_with_format(path, ImageFormat::Jpeg)?;

    Ok(CoverImage {
        width: i64::from(width),
        height: i64::from(height),
        version: "placeholder-jpeg-v1",
    })
}
