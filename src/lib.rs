#![feature(path_file_prefix)]

mod definition;
mod expr;
mod table;

use anyhow::Context as AnyhowContext;
pub use definition::Definition;
pub use expr::{Context, Expand, Expr, Lookup};
pub use table::{Record, Table};

use regex::{Captures, Regex};

use std::collections::HashMap;

pub fn pop(input: &str, def: Definition) -> anyhow::Result<Vec<String>> {
    let Definition { vars, defs } = def;

    let mut output = Vec::new();

    for variable in vars.iter() {
        let popped = pop_one(input, variable, &defs)?;
        output.push(popped);
    }

    Ok(output)
}

fn replace_all<E>(
    re: &Regex,
    haystack: &str,
    replacement: impl Fn(&Captures) -> Result<String, E>,
) -> Result<String, E> {
    let mut new = String::with_capacity(haystack.len());
    let mut last_match = 0;
    for caps in re.captures_iter(haystack) {
        let m = caps.get(0).unwrap();
        new.push_str(&haystack[last_match..m.start()]);
        new.push_str(&replacement(&caps)?);
        last_match = m.end();
    }
    new.push_str(&haystack[last_match..]);
    Ok(new)
}

pub fn pop_one(
    input: &str,
    record: &Record,
    defs: &HashMap<String, Table>,
) -> Result<String, anyhow::Error> {
    let re = Regex::new(r#"(?P<expr>\{\{[^}]+\}\})"#).unwrap();

    // replace all expr with their value in the definition
    let output = replace_all(
        &re,
        input,
        |captures: &Captures| -> anyhow::Result<String> {
            let matched = captures.name("expr").unwrap();
            let span = || {
                let rng = matched.range();
                format!("({}, {})", rng.start, rng.end)
            };

            let expression = matched.as_str();

            let expr = expression
                .parse::<Expr>()
                .with_context(|| format!("invalid expression: {expression} at {}", span()))?;

            expr.run(record, defs)
                .with_context(|| format!("unable to expand expression {expression} at {}", span()))
        },
    )?;

    Ok(output.to_string())
}
