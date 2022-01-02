use crate::types::*;

/// Convert a MEDLINE/Pubmed nbib (RIS-like) tag into corresponding CSL tag/value
///
/// The return type is a nullable algebreic type that supports ordinary types, name fields, and date fields
/// Recognized but non-supported tags, and unrecognized tags both yield an empty result: CSLValue(null)
/// On error, CSLValue(null) is also returned (TODO: consider expect package or similar)
///
/// Notes: the "note" field should only appear once; therefore the PMID and PMC (and a few others)
/// field handling is problematic
pub fn process_tag(tag: String, value: String) -> Result<CSLValue, String> {
    if tag.is_empty() || tag.chars().count() > 4 {
        return Err("MEDLINE/Pubmed nbib tags are 1-4 characters".into());
    }

    match &*tag {
        "AB" => Ok(CSLValue::CSLOrdinaryField(CSLOrdinaryField {
            key: "abstract".into(),
            value,
        })),
        "PMID" => Ok(CSLValue::CSLOrdinaryField(CSLOrdinaryField {
            key: "note".into(),
            value: format!("PMID: {}", value),
        })),
        "PMC" => Ok(CSLValue::CSLOrdinaryField(CSLOrdinaryField {
            key: "note".into(),
            value: format!("PMCID: {}", value),
        })),
        // Manuscript Identifier (MID) TODO
        "TI" => Ok(CSLValue::CSLOrdinaryField(CSLOrdinaryField {
            key: "title".into(),
            value,
        })),
        "VI" => Ok(CSLValue::CSLOrdinaryField(CSLOrdinaryField {
            key: "volume".into(),
            value,
        })),
        "IP" => Ok(CSLValue::CSLOrdinaryField(CSLOrdinaryField {
            key: "issue".into(),
            value,
        })),
        "PG" => Ok(CSLValue::CSLOrdinaryField(CSLOrdinaryField {
            key: "page".into(),
            value,
        })),
        // TODO need to transform to ISO 8601 per CSL specs;
        // medline looks like YYYY Mon DD
        "DP" => Ok(CSLValue::CSLDateField(CSLDateField::with_raw(
            "issued".into(),
            value,
        ))),

        "FAU" => Ok(CSLValue::CSLNameField(CSLNameField::with_name(
            "author".into(),
            value,
        ))),
        "AU" => Ok(CSLValue::CSLNameField(CSLNameField::with_name(
            "author".into(),
            value,
        ))),
        "FED" => Ok(CSLValue::CSLNameField(CSLNameField::with_name(
            "editor".into(),
            value,
        ))),
        "ED" => Ok(CSLValue::CSLNameField(CSLNameField::with_name(
            "editor".into(),
            value,
        ))),

        // This would typically be an ORCID
        // CSL name-variable definition does not have designated place for author id
        // https://github.com/citation-style-language/schema/blob/c2142118a0265dfcf7d66aa3328251bedcc66af2/schemas/input/csl-data.json#L463-L498
        //
        // Because this can appear in the middle of author lists, we will ignore it for now :/
        // instead of returning a "note" field -- however, once we have "note" aggregation,
        // and we can ensure it runs before author deduplication , OK to emit as a note
        "AUID" => Ok(CSLValue::None),

        // return CSL "language"
        // TODO, MEDLINE/Pubmed uses 3 letter language code; does CSL specify 3 or 2 letter?a
        // https://www.nlm.nih.gov/bsd/language_table.html
        "LA" => Ok(CSLValue::CSLOrdinaryField(CSLOrdinaryField {
            key: "language".into(),
            value,
        })),

        "SI" => Ok(CSLValue::CSLOrdinaryField(CSLOrdinaryField {
            key: "note".into(),
            value,
        })),

        // (GR) Grant Number

        // PT: Publication Type
        //
        // This field describes the type of material that the article represents;
        // it characterizes the nature of the information or the manner in which
        // it is conveyed (e.g., Review, Letter, Retracted Publication, Clinical Trial).
        // Records may contain more than one Publication Type, which are listed in alphabetical order.
        //
        // Almost all citations have one of these four basic, most frequently used
        // Publication Types applied to them: Journal Article, Letter, Editorial, News.
        // One of the above four Publication Types is applied to more than 99% of
        // all citations indexed for MEDLINE.
        //
        // Reference: https://www.nlm.nih.gov/mesh/pubtypes.html
        "PT" => {
            if value == "Journal Article" {
                // return "type": "article-journal"
                // https://aurimasv.github.io/z2csl/typeMap.xml#map-journalArticle
                Ok(CSLValue::CSLOrdinaryField(CSLOrdinaryField {
                    key: "type".into(),
                    value: "article-journal".into(),
                }))
            } else {
                Ok(CSLValue::None)
            }
        },

        "TA" => Ok(CSLValue::CSLOrdinaryField(CSLOrdinaryField{
            key: "container-title-short".into(),
            value,
        })),

        "JT" => Ok(CSLValue::CSLOrdinaryField(CSLOrdinaryField{
            key: "container-title".into(),
            value,
        })),

        "AID" => {
            // if DOI, return CSL "DOI" , and strip trailing "[doi]"
            match value.strip_suffix("[doi]") {
                Some(v) => Ok(CSLValue::CSLOrdinaryField(CSLOrdinaryField {
                    key: "DOI".into(),
                    value: v.to_string()
                })),
                None => Ok(CSLValue::None)
            }
        },

        _ => Ok(CSLValue::None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_badtag() {
        assert!(process_tag("XYZZY".into(), "val".into()).is_err());
    }

    #[test]
    fn test_unrecognized() {
        assert_eq!(
            process_tag("XYZ".into(), "val".into()).unwrap(),
            CSLValue::None,
        );
    }

    #[test]
    fn test_abstract() {
        let res = CSLOrdinaryField {
            key: "abstract".into(),
            value: "This is the abstract".into(),
        };
        assert_eq!(
            process_tag("AB".into(), "This is the abstract".into()).unwrap(),
            CSLValue::CSLOrdinaryField(res)
        );
    }

    #[test]
    fn test_pmid() {
        let res = CSLOrdinaryField {
            key: "note".into(),
            value: "PMID: 12345".into(),
        };
        assert_eq!(
            process_tag("PMID".into(), "12345".into()).unwrap(),
            CSLValue::CSLOrdinaryField(res)
        );
    }

    #[test]
    fn test_author() {
        let res = CSLNameField::with_name("author".into(), "Blachly, James S".into());
        assert_eq!(
            process_tag("FAU".into(), "Blachly, James S".into()).unwrap(),
            CSLValue::CSLNameField(res)
        );
    }
}