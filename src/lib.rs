//! Direct port of https://github.com/blachlylab/nbib/

#![feature(slice_group_by)]

mod tags;
mod transforms;
mod types;

pub fn transmogrify(mut input: impl std::io::Read) -> Result<String, ()>
{
    let mut buf = String::new();
    input.read_to_string(&mut buf).map_err(|_| ())?;
    let a: Vec<&str> = buf.lines()
        //.map(|l| l.trim_end())
        .collect();

    let _b: Vec<&[&str]> = a.split(|line| *line == "").collect();    // Vec of groups (slice) of lines
    let b = a.split(|line| *line == ""); // an iterator over groups of lines

    let c = b.map(|sl| sl.iter().cloned());  // iterator over iterator

    let d = c.map(transforms::merge_multiline_items).map(|x| x.into_iter());

    let e = d.map(transforms::medline_to_csl);

    // let f = e.map(transforms::reduce_authors);

    println!("{:?}", e);
    Ok("Sorry, Not Finished".into())
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

        transmogrify(input.as_bytes());
    }
}