//! Direct port of https://github.com/blachlylab/nbib/
use std::io::{self, prelude::*, BufReader};

mod tags;
mod transforms;
mod types;
mod groupby;

pub fn nbib_to_csl(mut input: impl std::io::Read) -> Result<String, String>
{
    let mut buf = String::new();
    input.read_to_string(&mut buf).map_err(|e|e.to_string())?;
    let a: Vec<&str> = buf.lines()
        //.map(|l| l.trim_end())
        .collect();

    let _b: Vec<&[&str]> = a.split(|line| *line == "").collect();    // Vec of groups (slice) of lines
    let b = a.split(|line| *line == ""); // an iterator over groups of lines

    let c = b.map(|sl| sl.iter().cloned());  // iterator over iterator

    let d = c.map(transforms::merge_multiline_items).map(|x| x.into_iter());

    let e = d.map(transforms::medline_to_csl);

    let f = e.map(|x| x.map(|y| y.unwrap())).map(transforms::reduce_authors);

    // println!("{:?}", f.map(|x| x.collect::<Vec<types::CSLValue>>()).collect::<Vec<Vec<types::CSLValue>>>());
    Ok(serde_json::to_string(&transforms::to_json(f).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?)
    // Ok("".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nil() {
        let mut input = String::from(r#"PMID- 12345
XY  - Unused field
AB  - This is the abstract's first line
      and this is its second line;
      with conclusion.
FAU - Blachly, James S
FAU - Gregory, Charles Thomas"#);

let expected = r#"
[
    {
        "abstract":"This is the abstract's first line and this is its second line; with conclusion.",
        "author":[
            {
                "family":"Blachly",
                "given":"James S"
            },
            {
                "family":"Gregory",
                "given":"Charles Thomas"
            }
        ],
        "id":"nbib-16023367964268412416",
        "note":"PMID: 12345"
    }
]"#;
        let e_json: serde_json::Value = serde_json::from_str(expected).unwrap();
        assert!(nbib_to_csl(input.as_bytes()).unwrap() == serde_json::to_string(&e_json).unwrap());
    }

    
    #[test]
    fn real_cite() {
        use std::fs::File;
        use std::io::BufReader;
        use std::path::PathBuf;
        let dir = env!("CARGO_MANIFEST_DIR");
        let f = File::open(PathBuf::from(dir).join("tests").join("fade.nbib")).unwrap();
        let e_f = File::open(PathBuf::from(dir).join("tests").join("fade.json")).unwrap();
        let exp = BufReader::new(e_f).lines().map(|x| x.unwrap()).collect::<Vec<String>>().join("");
        let e_json: serde_json::Value = serde_json::from_str(&exp).unwrap();
        assert!(nbib_to_csl(f).unwrap() == serde_json::to_string(&e_json).unwrap());
    }
}

