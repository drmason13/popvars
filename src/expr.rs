// use std::str::FromStr;

// #[allow(dead_code)]
// pub enum Expr {
//     Lookup(Lookup),
// }

// pub struct Lookup {
//     // key = variable.get(via.unwrap_or(table))
//     pub via: Option<String>,
//     // for table in path { key = resolve(key, table) }
//     pub path: Vec<String>,
//     // lookup.get(field)
//     pub field: String,
// }

// impl FromStr for Expr {
//     type Err = anyhow::Error;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         use parsely::*;

//         ?
//     }
// }

// mod parsing {
//     use parsely::*;

//     use super::*;

//     pub fn via() -> impl Parse<Output=String> {
//         until('@').map(str::to_string).then_skip(char('@'))
//     }

//     pub fn get_from(input: &str) -> ParseResult<(String, String)> {
//         let (from, remaining) = until('.').lex(input)?;

//         .many(0..).delimiter(char('.'))).optional()
//     }

//     pub fn lookup(input: &str) -> ParseResult<Lookup> {
//         let ((from, via), remaining) = via().optional() //
//             .then(

//             ).parse(input)?;

//         let ()

//         Ok(())
//     }
// }
