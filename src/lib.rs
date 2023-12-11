#![feature(path_file_prefix)]

mod definition;
mod expr;
mod table;

pub use definition::Definition;
pub use expr::{Context, Expand, Expr, Lookup};
pub use table::{Record, Table};

use regex::Regex;

use std::collections::HashMap;

pub fn pop(input: &str, def: Definition) -> Vec<String> {
    let Definition { vars, defs } = def;

    let mut output = Vec::new();

    for variable in vars.iter() {
        let popped = pop_one(input, variable, &defs);
        output.push(popped);
    }

    output
}

pub fn pop_one(input: &str, context: &Context, defs: &HashMap<String, Table>) -> String {
    let re = Regex::new(r#"\$\((?P<expr>[^\)]+?)\)"#).unwrap();

    let _ = (re, input, context, defs);

    // // replace all expr with their value in the definition
    // re.replace_all(input, |captures: &Captures| {
    //     let expr = captures
    //         .name("expr")
    //         .unwrap()
    //         .as_str()
    //         .parse::<Expression>();
    //     match var.get(&expr.to_lowercase()) {
    //         Some(value) => value.to_owned(),
    //         None => expr.to_owned(),
    //     }
    // })
    // .to_string()
    todo!()
}

#[cfg(test)]
pub(crate) mod test_utils;
