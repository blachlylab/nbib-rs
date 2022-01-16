use crate::tags::*;
use crate::types::*;
use super::groupby::GroupByItr;

pub struct MergeMultiline<'a, I>
where
    I: Iterator<Item = &'a str>,
{
    range: I,
    buf: Vec<String> // temporary buffer; holds rows that should be concat'd
}

impl<'a, I: Iterator> Iterator for MergeMultiline<'a, I> 
where
    I: Iterator<Item = &'a str>,
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        
                                       // into a single element in `ret`

        while let Some(row) = self.range.next() {
            assert!(row.chars().count() > 4, "Malformed record of length <= 4");
            if row.chars().nth(4).unwrap() == '-' && self.buf.is_empty() {
                // buf ~= row.strip;
                self.buf.push(row.trim().to_string());
            } else if row.chars().nth(4).unwrap() == '-' && !self.buf.is_empty() {
                let ret: String;
                // New record; buf may contain one or more rows
                // merge (if applicable) buf and append to ret
                if self.buf.len() == 1 {
                    // New record immediately after prior single-line record
                    ret = self.buf.pop().unwrap();
                    self.buf.clear();
                } else {
                    // New record after prior multi-line record
                    ret = self.buf.join(" "); // `trim()` removed trailing and leading spaces
                    self.buf.clear();
                }
                // then add current record to buf
                self.buf.push(row.trim().to_string());
                return Some(ret)
            } else if row.chars().nth(4).unwrap() != '-' && !self.buf.is_empty() {
                // A multi-line continuation
                //buf ~= row.strip;
                self.buf.push(row.trim().to_string())
            } else {
                panic!("Invalid State");
            }
        }

        // Now, buf may be empty if the last row was the end of a multi-line record (unlikely)
        // but to be safe we must test it is nonempty before finally dumping it to ret
        if self.buf.is_empty() {
            // noop
            return None
        } else if self.buf.len() == 1 {
            return Some(self.buf.pop().unwrap())
        } else {
            return Some(self.buf.join(" "))
        }
    }
}

/// Merge multi-line records from a range of strings lazily
///
/// For example:
/// ["AB  - Abstract first line...", "      continued second..."]
/// would be merged into a single record in the output range
/// The complete range might look like:
/// ["PMID- 12345", "TI  - Article title", "AB  - Abstr line 1", "      ...line2", "AU  - Blachly JS"]
pub fn merge_multiline_items<'a, I>(range: I) -> MergeMultiline<'a, I>
where
    I: Iterator<Item = &'a str>,
{
    MergeMultiline {range: range, buf: vec![]}
}

/// Convert medline record (group of tags) to CSL-JSON item tags lazily
pub fn medline_to_csl<'a, I>(range: I) -> impl Iterator<Item = Result<CSLValue, String>> 
where
    I: Iterator<Item = String>,
{
    range.map(|row|{
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

        process_tag(key, value)
    }).filter(|x| {
        match x {
            Ok(x) => {
                match x {
                    CSLValue::None => false,
                    _ => true
                }
            }
            Err(_) => true
        }
    })
}


/// Merge author records when both FAU and AU appear for same author
/// (or make best effort)
///
/// Takes a range of CSL tags/values (collectively, a complete record => rec)
///
/// TODO: support multiple types (author, editor)
pub fn reduce_authors<I>(range: I) -> impl Iterator<Item = CSLValue> 
where
    I: Iterator<Item = CSLValue>,
{
    // let names = range.filter(|x| );

    let names_grouped_by_type = range.group_by(|a, b| a.is_name() && b.is_name() && a.key() == b.key());

    /*    auto reduced = namesGroupedByType
        .map!(n => n.chunkBy!(
            (a,b) => a.tryGetMember!"np".family.split(" ")[0] ==
                     b.tryGetMember!"np".family.split(" ")[0])
            .map!(y => y.takeOne)
        ).joiner.joiner;
    */
    let grouped_by_family = names_grouped_by_type
        .map(|n| n.group_by(
                |a, b|  a.is_name() && b.is_name() && 
                        a.np().unwrap().family.as_ref().unwrap().split(" ").nth(0) ==
                        b.np().unwrap().family.as_ref().unwrap().split(" ").nth(0)
                )
        );

    let reduced = grouped_by_family.map(|x| x.map(|y| y.into_iter().nth(0).unwrap()));
    let reduced = reduced.flatten();
    // `reduced` now contains deduplicated names

    // let no_names = rec.iter().filter(|x| !x.is_name());
    // D:   return chain(noNames, reduced);
    reduced//.map(|x| x.clone()).collect()
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

        let merged_rec: Vec<String> = merge_multiline_items(rec.clone().into_iter()).collect();

        assert_eq!(merged_rec.len(), 7);
        assert_eq!(
            merged_rec[2],
            "AB  - This is the abstract's first line and this is its second line; with conclusion."
        );

        let csl = medline_to_csl(merged_rec.into_iter());
        let names: Vec<CSLValue> = csl.map(|x| x.unwrap()).filter(|x| x.is_name()).collect();
        assert_eq!(names.len(), 4);

        let merged_rec = merge_multiline_items(rec.into_iter());

        let reduced = reduce_authors(medline_to_csl(merged_rec.into_iter()).map(|x| x.unwrap()));
        let names: Vec<CSLValue> = reduced.filter(|x| x.is_name()).collect();
    
        assert_eq!(names.len(), 2);
        assert_eq!(names[0].key().unwrap(), "author");
        assert_eq!(names[0].np().unwrap().family, Some(String::from("Blachly")));
        assert_eq!(names[0].np().unwrap().given, Some(String::from("James S")));
        assert_eq!(names[1].key().unwrap(), "author");
        assert_eq!(names[1].np().unwrap().family, Some(String::from("Gregory")));
        assert_eq!(names[1].np().unwrap().given, Some(String::from("Charles Thomas")));    
    }
}
