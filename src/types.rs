/// CSL item record
///
/// Tags and values are stored in one of three arrays, according to whether they are ordinary, name, or date.
/// Serialization to JSON is implemented manually to keep contained `tag: value` at top level (non-nested)
/// A few transformations are made:
///     (1) `id` is injected (and any existing id, which shouldn't happen, is ignored)
///     (2) name field tags are aggregated and grouped by type (author, editor, etc.)
///     (3) ???
///
/// The end result should be semantically-correct CSL-JSON
///
/// Reference: https://citeproc-js.readthedocs.io/en/latest/csl-json/markup.html
/// Reference: https://github.com/citation-style-language/schema/blob/master/schemas/input/csl-data.json
pub struct CSLItem {
    fields: Vec<CSLOrdinaryField>,
    names: Vec<CSLNameField>,
    dates: Vec<CSLDateField>,
}

/// CSL-JSON value
///
/// Specification defines them as ordinary fields, name fields, or date fields
/// We additionally allow None/null as signal for our conversion program taht
/// either something went wrong or that a tag was ignored in conversion
/// D: alias CSLValue = Nullable!(CSLOrdinaryField, CSLNameField, CSLDateField);
#[derive(Debug, PartialEq)]
pub enum CSLValue {
    None,
    CSLOrdinaryField(CSLOrdinaryField),
    CSLNameField(CSLNameField),
    CSLDateField(CSLDateField),
}

//#[derive(Serialize)]
#[derive(Debug, PartialEq)]
pub struct CSLOrdinaryField {
    pub key: String,
    pub value: String,
}

/// CSL "name" field
/// 
/// Reference:
/// https://github.com/citation-style-language/schema/blob/c2142118a0265dfcf7d66aa3328251bedcc66af2/schemas/input/csl-data.json#L463-L498
#[derive(Debug, PartialEq)]
pub struct CSLNameField {
    pub key: String,

    /// "Full" author, editor, etc. name; in MEDLINE/Pubmed format
    /// corresponds to "FAU" or "FED"
    pub full: bool,

    pub np: NameParts,
}

impl CSLNameField {
    pub fn with_name(nametype: String, name: String) -> Self {

        let name_parts: Vec<&str> = name.split(",").collect();
        let mut np = NameParts::default();
        match name_parts.len() {
            0 => panic!(),
            1 => np.family = Some(name.trim().to_string()),
            2 => {
                np.family = Some(name_parts[0].trim().to_string());
                np.given = Some(name_parts[1].trim().to_string());
            },
            _ => {
                // > 1 comma; perhaps from a suffix like ", Jr."
                // TODO: Warn? 
                np.family = Some(name.trim().to_string());
            }
        };

        CSLNameField {
            key: nametype,
            full: if name_parts.len() == 2 {true} else {false},
            np
        }
    }
}

/// Embedded in CSLNameField
#[derive(Debug, Default, PartialEq)]
pub struct NameParts {
    pub family: Option<String>,
    
    pub given: Option<String>,
    
    // @serdeKeys("dropping-particle")
    pub dropping_particle: Option<String>,
    
    // @serdeKeys("non-dropping-particle")
    pub non_dropping_particle: Option<String>,
    
    pub suffix: Option<String>,

    //         @serdeKeys("comma-suffix")
    pub comma_suffix: Option<String>,

    //         @serdeKeys("static-ordering")
    pub static_ordering: Option<String>,

    pub literal: Option<String>,

    //        @serdeKeys("parse-names")
    pub parse_names: Option<String>,
}

/// CSL "date" field
/// 
/// Reference:
/// https://github.com/citation-style-language/schema/blob/c2142118a0265dfcf7d66aa3328251bedcc66af2/schemas/input/csl-data.json#L505-L546
#[derive(Debug, PartialEq)]
pub struct CSLDateField {
    pub key: String,

    pub dp: DateParts,
}

impl CSLDateField {
    pub fn with_raw(key: String, raw: String) -> Self {
        let mut dp = DateParts::default();
        dp.raw = Some(raw);
        CSLDateField{
            key,
            dp
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct DateParts {
    /*
        @serdeKeys("date-parts")
        @serdeIgnoreOutIf!empty
        string date_parts;

        @serdeIgnoreOutIf!empty
        string season;

        @serdeIgnoreOutIf!empty
        string circa;   // string, number, bool

        @serdeIgnoreOutIf!empty
        string literal;

        @serdeIgnoreOutIf!empty
        string raw;

        @serdeIgnoreOutIf!empty
        string edtf;
    */
    date_parts: Option<String>,
    season: Option<String>,
    circa: Option<String>,  // String, number, bool
    literal: Option<String>,
    raw: Option<String>,
    edtf: Option<String>,
}