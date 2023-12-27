#![feature(path_file_prefix)]

mod definition;
mod expr;
mod table;
mod template;

use anyhow::Context as AnyhowContext;
pub use definition::Definition;
pub use expr::{Context, Expand, Expr, Lookup};
pub use table::{Record, Table};
pub use template::Template;

pub fn pop(input: &str, def: Definition) -> anyhow::Result<Vec<String>> {
    let mut output = Vec::new();

    let template = Template::compile(input)?;

    dbg!(&template);

    for (n, var) in def.vars.iter().enumerate() {
        let popped = template.pop(var, &def).with_context(|| {
            format!("Error while populating template with row {} of vars", n + 1)
        })?;
        output.push(popped);
    }

    Ok(output)
}
