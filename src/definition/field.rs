/// A Field is either a special field or just a plain string.
///
/// Fields are used to reference [`Value`]s when rendering.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Field {
    Special(SpecialField),
    Plain(String),
}

/// Special fields are used for all kinds of things, most notably the Id ($id) special field is used to identify instances of a type.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum SpecialField {
    Id,
    OutFile,
}

impl Field {
    pub fn new(s: &str) -> Self {
        match s {
            // special fields
            "$id" => Field::Special(SpecialField::Id),
            "$outfile" => Field::Special(SpecialField::OutFile),

            // all other fields are plain until proven to be a Type
            _ => Field::Plain(s.to_owned()),
        }
    }
}

impl AsRef<str> for Field {
    fn as_ref(&self) -> &str {
        match self {
            Field::Special(s) => s.as_ref(),
            Field::Plain(s) => s.as_ref(),
        }
    }
}

impl AsRef<str> for SpecialField {
    fn as_ref(&self) -> &str {
        match self {
            SpecialField::Id => "$id",
            SpecialField::OutFile => "$outfile",
        }
    }
}
