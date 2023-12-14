//! Contains types and methods to compile templates from strings into a kind of execution plan
//! to populate the template
//!

use std::collections::HashMap;

use crate::{expr::WhereClause, table::Value, Context, Definition, Expand, Record};

/// [`Template`]s consists of an ordered list of [`Node`]s to be rendered in order and a
/// [`ContextIndex`] to access the required [Context] for populating
pub struct Template {
    nodes: Vec<Node>,
    /// provides a way to access / compute the required context to fill in the Template
    ///
    /// (note: vars and defs are always required and not included,
    /// so the root template will have an empty Context)
    context: Option<AdditionalContext>,
}

impl Template {
    pub fn compile(input: &str) -> anyhow::Result<Self> {
        todo!()
    }

    pub fn pop(&self, record: &Record, def: &Definition) -> anyhow::Result<String> {
        let mut output = String::new();

        for node in self.nodes.iter() {
            node.pop(&mut output, record, def, &self.context)?;
        }

        Ok(output)
    }
}

pub type AdditionalContext = HashMap<String, ContextIndex>;

/// [`ContextIndex`] can be used to [`index()`] [`Definition`] to return &[Context] to use while populating a [`Template`]
///
/// [`index()`]: Definition::index
pub enum ContextIndex {
    /// A single Value inline rather than an Index as such
    Value(Value),

    /// A list of values inline, rather than an Index as such
    ValueList(Vec<Value>),

    /// selects an entire Table to provide as context
    Table(String),

    /// A Where Clause selects a filtered list of Records from a Table to provide as context
    FilteredTable {
        context_name: String,
        table_name: String,
        where_clause: Option<WhereClause>,
    },
}

pub enum Node {
    Text(String),
    Expand(Expand),
    NestedTemplate(Template),
}

impl Node {
    fn run<'c>(
        &self,
        context: &'c Context,
        def: &'c Definition,
        additional_context: &AdditionalContext,
    ) -> anyhow::Result<String> {
        todo!()
    }

    fn pop(
        &self,
        output: &mut String,
        record: &Record,
        def: &Definition,
        context: &Option<AdditionalContext>,
    ) -> anyhow::Result<()> {
        todo!()
    }
}
