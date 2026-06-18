//! Pick the answer key to grade one sheet against, for a mixed-variant batch.
//!
//! A scanned stack may interleave variants A/B/C…. Each [`ParsedSheet`] carries
//! the variant decoded from its bubble row
//! ([`ParsedSheet::variant`](crate::domain::ParsedSheet::variant)); this selector
//! turns that — plus a manual fallback — into the [`AnswerKey`] the engine scores
//! the sheet with.
//!
//! Resolution rule:
//! - **Detected variant present** → grade against the key whose `variant` matches
//!   it. If no such key exists, return `None` — the student declared a form the
//!   exam has no key for, which must go to manual review rather than be silently
//!   graded against the wrong key.
//! - **No detected variant** (template has no variant row, or the mark was
//!   blank/ambiguous) → fall back to `fallback_variant`'s key. This is the
//!   single-variant path: the teacher names one variant in `/grade` and every
//!   sheet uses it.
//! - Nothing resolves → `None`. The caller flags the sheet `needs_review`.
//!
//! Matching is exact string equality on the variant name, which is why variant
//! names are constrained to the printed bubble letters (A/B/C…).

use crate::domain::AnswerKey;

/// Select the answer key for one sheet. See the module docs for the rule.
pub fn select_answer_key<'a>(
    keys: &'a [AnswerKey],
    detected_variant: Option<&str>,
    fallback_variant: Option<&str>,
) -> Option<&'a AnswerKey> {
    match detected_variant {
        // A confidently-detected variant binds to its own key only. A miss here
        // is a review case, never a fallback — grading a "C" sheet against the
        // "A" key would silently mis-score the whole page.
        Some(v) => keys.iter().find(|k| k.variant == v),
        // No reliable detection: use the manually chosen variant, if any.
        None => fallback_variant.and_then(|v| keys.iter().find(|k| k.variant == v)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{AnswerKey, AnswerKeyEntry};

    fn key(variant: &str) -> AnswerKey {
        AnswerKey {
            exam_id: 1,
            variant: variant.to_string(),
            answers: vec![AnswerKeyEntry {
                group_id: "q1".into(),
                correct_indices: vec![0],
            }],
        }
    }

    #[test]
    fn detected_variant_binds_to_its_own_key() {
        let keys = vec![key("A"), key("B"), key("C")];
        let chosen = select_answer_key(&keys, Some("B"), None).unwrap();
        assert_eq!(chosen.variant, "B");
    }

    #[test]
    fn detected_variant_wins_over_fallback() {
        let keys = vec![key("A"), key("B")];
        let chosen = select_answer_key(&keys, Some("B"), Some("A")).unwrap();
        assert_eq!(chosen.variant, "B");
    }

    #[test]
    fn detected_variant_with_no_matching_key_does_not_fall_back() {
        // Student bubbled "C" but the exam has no "C" key — must be review, not
        // a silent mis-grade against the fallback.
        let keys = vec![key("A"), key("B")];
        assert!(select_answer_key(&keys, Some("C"), Some("A")).is_none());
    }

    #[test]
    fn no_detection_uses_fallback() {
        let keys = vec![key("A"), key("B")];
        let chosen = select_answer_key(&keys, None, Some("A")).unwrap();
        assert_eq!(chosen.variant, "A");
    }

    #[test]
    fn no_detection_and_no_fallback_yields_none() {
        let keys = vec![key("A")];
        assert!(select_answer_key(&keys, None, None).is_none());
    }

    #[test]
    fn no_detection_with_unmatched_fallback_yields_none() {
        let keys = vec![key("A")];
        assert!(select_answer_key(&keys, None, Some("Z")).is_none());
    }

    #[test]
    fn empty_key_set_yields_none() {
        let keys: Vec<AnswerKey> = vec![];
        assert!(select_answer_key(&keys, Some("A"), Some("A")).is_none());
    }
}
