//! Cross-language parity test for the `OmrTemplate` JSON shape (Rule 3).
//!
//! Generates a canonical fixture from the Rust domain types and asserts it
//! round-trips through `serde_json` byte-for-byte. The same fixture is loaded
//! by the TS Vitest suite (`src/lib/types/template.test.ts`) and parsed
//! through the Zod `templateSchema` to confirm field-name parity between the
//! Rust struct and the TypeScript Zod definition.
//!
//! When the schema changes, regenerate the fixture by running:
//!     cargo test --test template_serde -- --ignored regenerate_fixture
//! Then commit the resulting `tests/fixtures/template-v1.json`.

use std::path::PathBuf;

use shalgalt_omr_lib::domain::template::{
    BubbleGroup, BubbleKind, Marker, OmrTemplate, TemplatePoint,
};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("template-v1.json")
}

fn canonical_template() -> OmrTemplate {
    OmrTemplate {
        version: OmrTemplate::CURRENT_VERSION,
        title: "Жишээ загвар".to_string(),
        markers: [
            Marker {
                id: "m-tl".into(),
                position: TemplatePoint { x: 0.05, y: 0.05 },
                size: 0.02,
            },
            Marker {
                id: "m-tr".into(),
                position: TemplatePoint { x: 0.95, y: 0.05 },
                size: 0.02,
            },
            Marker {
                id: "m-br".into(),
                position: TemplatePoint { x: 0.95, y: 0.95 },
                size: 0.02,
            },
            Marker {
                id: "m-bl".into(),
                position: TemplatePoint { x: 0.05, y: 0.95 },
                size: 0.02,
            },
        ],
        groups: vec![
            BubbleGroup {
                id: "grp-sid".into(),
                kind: BubbleKind::StudentId,
                label: "Сурагчийн дугаар".into(),
                bubbles: vec![
                    TemplatePoint { x: 0.10, y: 0.20 },
                    TemplatePoint { x: 0.15, y: 0.20 },
                    TemplatePoint { x: 0.20, y: 0.20 },
                ],
                answer_index: None,
                score: 1.0,
                section: None,
            },
            BubbleGroup {
                id: "grp-q1".into(),
                kind: BubbleKind::Question,
                label: "Q1".into(),
                bubbles: vec![
                    TemplatePoint { x: 0.30, y: 0.40 },
                    TemplatePoint { x: 0.35, y: 0.40 },
                    TemplatePoint { x: 0.40, y: 0.40 },
                    TemplatePoint { x: 0.45, y: 0.40 },
                    TemplatePoint { x: 0.50, y: 0.40 },
                ],
                answer_index: Some(2),
                score: 1.0,
                section: None,
            },
        ],
    }
}

#[test]
fn fixture_matches_committed_file() {
    let template = canonical_template();
    let serialized = serde_json::to_string_pretty(&template).expect("serialize template");

    let committed = std::fs::read_to_string(fixture_path()).expect(
        "tests/fixtures/template-v1.json missing — regenerate via `cargo test --ignored regenerate_fixture`",
    );

    assert_eq!(
        committed.trim(),
        serialized.trim(),
        "Rust serialization no longer matches committed fixture. Either revert the schema \
         change or regenerate the fixture and update the TS Zod schema accordingly."
    );
}

#[test]
fn template_round_trips_through_serde() {
    let template = canonical_template();
    let s = serde_json::to_string(&template).expect("serialize");
    let parsed: OmrTemplate = serde_json::from_str(&s).expect("deserialize");
    let s2 = serde_json::to_string(&parsed).expect("re-serialize");
    assert_eq!(s, s2);
}

#[test]
#[ignore = "regenerate the committed fixture only when the schema intentionally changes"]
fn regenerate_fixture() {
    let template = canonical_template();
    let serialized = serde_json::to_string_pretty(&template).expect("serialize template");
    let path = fixture_path();
    std::fs::create_dir_all(path.parent().unwrap()).expect("mkdir");
    std::fs::write(&path, serialized).expect("write fixture");
    eprintln!("wrote {}", path.display());
}
