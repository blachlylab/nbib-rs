use serde::Serialize;
use serde::ser::{Serializer, SerializeSeq, SerializeMap};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
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
#[derive(Hash)]
pub struct CSLItem {
    pub fields: Vec<CSLOrdinaryField>,
    pub names: Vec<CSLNameField>,
    pub dates: Vec<CSLDateField>,
}

impl CSLItem {
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            names: Vec::new(),
            dates: Vec::new()
        }
    }
    fn calculate_id(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }

    fn name_types(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.names.iter()
            .map(|x| x.key.as_ref())
            .collect();
        names.sort();
        names.dedup();
        names
    }
}

impl Serialize for CSLItem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("id", &format!("nbib-{}", self.calculate_id()))?;
        for f in &self.fields {
            if f.key == "id" {
                continue;    // already injected id
            }
            //serializer.putKey(f.key);
            //serializer.putValue(f.value);
            map.serialize_entry(&f.key, &f.value)?;
        }
        let types = self.name_types();
        for t in types {
            let matchingNames = self.names.iter()
                .filter(|a| a.key == t)
                .map(|n| &n.np)
                .collect::<Vec<&NameParts>>();
            map.serialize_entry(t, &matchingNames)?;
        }
        for d in &self.dates {
            map.serialize_entry(&d.key, &d.dp)?;
        }
        map.end()
    }
}

/// CSL-JSON value
///
/// Specification defines them as ordinary fields, name fields, or date fields
/// We additionally allow None/null as signal for our conversion program taht
/// either something went wrong or that a tag was ignored in conversion
/// D: alias CSLValue = Nullable!(CSLOrdinaryField, CSLNameField, CSLDateField);
#[derive(Clone, Debug, PartialEq)]
pub enum CSLValue {
    None,
    CSLOrdinaryField(CSLOrdinaryField),
    CSLNameField(CSLNameField),
    CSLDateField(CSLDateField),
}

impl CSLValue {
    pub fn is_name(&self) -> bool {
        match self {
            Self::None => false,
            Self::CSLOrdinaryField(_) => false,
            Self::CSLNameField(_) => true,
            Self::CSLDateField(_) => false
        }
    }
    pub fn key(&self) -> Option<&str> {
        match self {
            Self::None => None,
            Self::CSLOrdinaryField(v) => Some(&v.key),
            Self::CSLNameField(v) => Some(&v.key),
            Self::CSLDateField(v) => Some(&v.key), 
        }
    }
    pub fn np(&self) -> Option<&NameParts> {
        match self {
            Self::None => None,
            Self::CSLOrdinaryField(_) => None,
            Self::CSLNameField(v) => Some(&v.np),
            Self::CSLDateField(_) => None,
        }
    }
}

//#[derive(Serialize)]
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct CSLOrdinaryField {
    pub key: String,
    pub value: String,
}

/// CSL "name" field
/// 
/// Reference:
/// https://github.com/citation-style-language/schema/blob/c2142118a0265dfcf7d66aa3328251bedcc66af2/schemas/input/csl-data.json#L463-L498
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct CSLNameField {
    pub key: String,

    /// "Full" author, editor, etc. name; in MEDLINE/Pubmed format
    /// corresponds to "FAU" or "FED"
    pub full: bool,

    pub np: NameParts,
}

impl CSLNameField {
    pub fn with_name(nametype: String, name: String) -> Self {

        let name_parts: Vec<&str> = name.split(',').collect();
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
            full: name_parts.len() == 2,
            np
        }
    }
}

/// Embedded in CSLNameField
#[derive(Clone, Debug, Default, PartialEq, Serialize, Hash)]
pub struct NameParts {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub given: Option<String>,
    
    #[serde(rename = "dropping-particle")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dropping_particle: Option<String>,
    
    #[serde(rename = "non-dropping-particle")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_dropping_particle: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,

    #[serde(rename = "comma-suffix")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comma_suffix: Option<String>,

    #[serde(rename = "static-ordering")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub static_ordering: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub literal: Option<String>,

    #[serde(rename = "parse-names")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_names: Option<String>,
}

/// CSL "date" field
/// 
/// Reference:
/// https://github.com/citation-style-language/schema/blob/c2142118a0265dfcf7d66aa3328251bedcc66af2/schemas/input/csl-data.json#L505-L546
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct CSLDateField {
    pub key: String,

    pub dp: DateParts,
}

impl CSLDateField {
    pub fn with_raw(key: String, raw: String) -> Self {
        let dp = DateParts { raw: Some(raw), ..Default::default() };
        CSLDateField{
            key,
            dp
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Hash)]
pub struct DateParts {
    #[serde(skip_serializing_if = "Option::is_none")]
    date_parts: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    season: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    circa: Option<String>,  // String, number, bool
    #[serde(skip_serializing_if = "Option::is_none")]
    literal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    edtf: Option<String>,
}