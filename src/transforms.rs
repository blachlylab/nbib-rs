use crate::types::*;

/// Merge multi-line records from a range of strings
/// (Unfortunately, not lazily)
///
/// For example:
/// ["AB  - Abstract first line...", "      continued second..."]
/// would be merged into a single record in the output range
/// The complete range might look like:
/// ["PMID- 12345", "TI  - Article title", "AB  - Abstr line 1", "      ...line2", "AU  - Blachly JS"]
pub fn merge_multiline_items<'a, I>(range: I) -> Vec<String>
where
    I: Iterator<Item = &'a str>,
{
    let mut ret = vec![];
    let mut buf: Vec<String> = vec![]; // temporary buffer; holds rows that should be concat'd
                                       // into a single element in `ret`

    for row in range {
        assert!(row.chars().count() > 4, "Malformed record of length <= 4");
        if row.chars().nth(4).unwrap() == '-' && buf.is_empty() {
            // buf ~= row.strip;
            buf.push(row.trim().to_string());
        } else if row.chars().nth(4).unwrap() == '-' && !buf.is_empty() {
            // New record; buf may contain one or more rows
            // merge (if applicable) buf and append to ret
            if buf.len() == 1 {
                // New record immediately after prior single-line record
                ret.push(buf.pop().unwrap());
                buf.clear();
            } else {
                // New record after prior multi-line record
                ret.push(buf.join(" ")); // `trim()` removed trailing and leading spaces
                buf.clear();
            }
            // then add current record to buf
            buf.push(row.trim().to_string());
        } else if row.chars().nth(4).unwrap() != '-' && !buf.is_empty() {
            // A multi-line continuation
            //buf ~= row.strip;
            buf.push(row.trim().to_string())
        } else {
            panic!("Invalid State");
        }
    }

    // Now, buf may be empty if the last row was the end of a multi-line record (unlikely)
    // but to be safe we must test it is nonempty before finally dumping it to ret
    if buf.len() == 0 {
        // noop
    } else if buf.len() == 1 {
        ret.push(buf.pop().unwrap());
    } else {
        ret.push(buf.join(" "));
    }

    ret
}

/// Convert medline record (group of tags) to CSL-JSON item tags
/// TODO make lazy
pub fn medline_to_CSL<'a, I>(range: I) -> Vec<CSLValue>
{
    todo!();
}

/// Merge author records when both FAU and AU appear for same author
/// (or make best effort)
/// 
/// Takes a range of CSL tags/values (collectively, a complete record => rec)
///
/// TODO: support multiple types (author, editor)
pub fn reduce_authors<'a, I>(rec: I) -> ()
{
    todo!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge() {
        let rec = vec![
            "PMID- 12345",
            "XY  - Unused field",
            "AB  - This is the abstract's first line",
            "      and this is its second line;",
            "      with conclusion.",
            "FAU - Blachly, James S",
            "FAU - Gregory, Charles Thomas",
        ];

        let merged_rec = merge_multiline_items(rec.into_iter());

        assert_eq!(merged_rec.len(), 5);
        assert_eq!(
            merged_rec[2],
            "AB  - This is the abstract's first line and this is its second line; with conclusion."
        );
    }
}
