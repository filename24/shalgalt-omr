//! One-shot example: render the Mongolian-standard preset with the same options the
//! desktop Tauri command (`pdf_generate_omr`) would use, then write it to a file.
//!
//! Usage (from repo root):
//!     cargo run -p shalgalt-pdf --example render_standard
//!
//! Default output path is `crates/shalgalt-pdf/tests/golden_data/mongolian_standard_full.pdf`
//! — alongside the byte-comparison `mongolian_standard.pdf` golden but distinct: the
//! golden renders the bare preset for deterministic byte-comparison, this file renders
//! with full production options (header + sidebar + manual underlines) so the user can
//! preview what the desktop "Save as PDF" command actually emits. Pass an explicit path
//! as the first arg to override.
//!
//! Pulls the preset from the test fixture module so the coordinates always match the
//! `golden.rs` regression. Production-equivalent options injected here:
//!   - title in Mongolian Cyrillic
//!   - sidebar instructions string (matches `apps/desktop/src/commands/pdf.rs`)
//!   - choice_labels A–E, digit_labels 0–9
//!   - manual_id_slots = true

#[path = "../tests/common/preset.rs"]
mod preset;

use shalgalt_pdf::{render_template, BubbleStyle, HeaderText, PdfOptions};
use std::path::PathBuf;

const DEFAULT_INSTRUCTIONS_MN: &str = "1. Хариултын хуудсын нугалж гэмтээж болохгүй.\n\
    2. Та шалгалтын дэвтрийн хувилбар, хариултын хуудасны хувилбар таалч буй \
    эсэхийг шалгана уу.\n\
    3. Хариултын хуудсыг зөвхөн балын харандаа болон хар, цэнхэр өнгийн үзгэн, \
    тосон балаар тод завсаргүй байдлаар бөглөнө үү.";

fn main() {
    let default_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden_data/mongolian_standard_full.pdf");
    let out_path: PathBuf = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or(default_path);

    let template = preset::mongolian_standard("Шалгалтын хариултын хуудас");

    let opts = PdfOptions {
        header: HeaderText {
            title: Some("Шалгалтын хариултын хуудас".to_string()),
            subtitle: Some("Жишээ загвар".to_string()),
            school: Some("Сургууль: ____________".to_string()),
            teacher: None,
        },
        instructions: Some(DEFAULT_INSTRUCTIONS_MN.to_string()),
        bubble_style: BubbleStyle::default(),
        ..PdfOptions::default()
    };

    let bytes = render_template(&template, &opts).expect("render");
    std::fs::write(&out_path, &bytes).expect("write");
    println!("wrote {} ({} bytes)", out_path.display(), bytes.len());
}
