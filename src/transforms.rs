use crate::tags::*;
use crate::types::*;

// Bring trait into scope
//use itertools::Itertools;

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
    if buf.is_empty() {
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
pub fn medline_to_csl<'a, I>(range: I) -> Result<Vec<CSLValue>, String>
where
    I: Iterator<Item = String>,
{
    let mut ret: Vec<CSLValue> = vec![];
    for row in range {
        // Format: "XXXX- The quick brown fox jumped..."
        // where XXXX of length 1-4 and right-padded
        assert!(row.chars().count() >= 7, "Malformed record");
        assert!(
            row.chars().collect::<Vec<char>>()[4] == '-',
            "Malformed record"
        );
        // TODO: Change the above to emit warning, and `continue`

        let key = row
            .chars()
            .take(4)
            .collect::<String>()
            .trim_end()
            .to_string();
        let value = row.chars().skip(6).collect::<String>();

        let csl = process_tag(key, value)?;
        match csl {
            CSLValue::None => {}
            CSLValue::CSLOrdinaryField(_) => ret.push(csl),
            CSLValue::CSLNameField(_) => ret.push(csl),
            CSLValue::CSLDateField(_) => ret.push(csl),
        };
    }
    Ok(ret)
}


/// Merge author records when both FAU and AU appear for same author
/// (or make best effort)
///
/// Takes a range of CSL tags/values (collectively, a complete record => rec)
///
/// TODO: support multiple types (author, editor)
pub fn reduce_authors(rec: Vec<CSLValue>) -> Vec<CSLValue> {
    let names = rec
        .iter()
        .filter(|x| x.is_name())
        //.cloned()
        //.collect::<Vec<CSLValue>>()
        .collect::<Vec<&CSLValue>>();
    let names_grouped_by_type = names.group_by(|a, b| a.key() == b.key());

    /*    auto reduced = namesGroupedByType
        .map!(n => n.chunkBy!(
            (a,b) => a.tryGetMember!"np".family.split(" ")[0] ==
                     b.tryGetMember!"np".family.split(" ")[0])
            .map!(y => y.takeOne)
        ).joiner.joiner;
    */
    let reduced = names_grouped_by_type
        .map(|n| n.group_by(
            |a, b|  a.np().unwrap().family.as_ref().unwrap().split(" ").nth(0) ==
                    b.np().unwrap().family.as_ref().unwrap().split(" ").nth(0))
            .map(|y| y[0])
        ).flatten();
    // `reduced` now contains deduplicated names

    let no_names = rec.iter().filter(|x| !x.is_name());
    // D:   return chain(noNames, reduced);
    no_names.chain(reduced).map(|x| x.clone()).collect()
    //no_names.chain(names_grouped_by_type).collect()
    //no_names.append(&mut reduced);

    //no_names
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
            "AU  - Blachly JS",
            "FAU - Gregory, Charles Thomas",
            "AU  - Gregory CT",
        ];

        let merged_rec = merge_multiline_items(rec.into_iter());

        assert_eq!(merged_rec.len(), 7);
        assert_eq!(
            merged_rec[2],
            "AB  - This is the abstract's first line and this is its second line; with conclusion."
        );

        let csl = medline_to_csl(merged_rec.into_iter()).unwrap();

        let reduced = reduce_authors(csl);
        let names: Vec<&CSLValue> = reduced.iter().filter(|x| x.is_name()).collect();
        assert_eq!(names.len(), 2);
    }
}
