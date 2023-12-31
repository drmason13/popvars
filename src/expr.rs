use anyhow::{anyhow, Context as _};
use parsely::{result_ext::*, Parse};
use std::{collections::HashMap, str::FromStr};

mod parsing;
pub use parsing::template;
use parsing::{expr, for_tag};

use crate::{
    template::{ContextIndex, InheritedContext},
    Record, Table,
};

/// [`Expr`] is exactly what is contained within `{{ }}` braces.
///
/// See also [`BlockExpr`] for what is contained with *opening* `{@ ___ @}` braces.
///
/// Each [`BlockExpr`] opens a [`Block`] which closes with a corresponding `{@ end ___ @} braces`
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Expand(Expand),
}

/// A [`Block`] is a [`BlockExpr`] paired with some inner content made up of [`Node`]s
#[derive(Clone, PartialEq, Debug)]
pub struct Block {
    pub expr: BlockExpr,
    pub nodes: Vec<Node>,
}

/// Each [`Node`] is a plain [text](Node::Text) [`String`], an (inline) [`Expr`], or a [`Block`].
/// Each template file is parsed into a [`Vec<Node>`] before being compiled into a [`Template`](crate::Template)
#[derive(Clone, PartialEq, Debug)]
pub enum Node {
    Text(String),
    Expr(Expr),
    Block(Block),
}

impl Node {
    pub fn from_text(text: &str) -> Self {
        Node::Text(text.into())
    }
}

/// Context and Record are interchangable, they are both the exact same type.
///
/// Context is used specifically in the context (no pun intended) of [lookup]s
/// where the "current context", i.e. the current record being used to [expand] an
/// [expression], can be swapped for a different one from another [table].
///
/// This "context-switching" is the mechanism for [expand]ing [expression]s in a
/// template using the following dot syntax:
///
/// ```bash
/// {{country.team}}
/// ```
///
/// This does a [lookup] in the `country` [table] and pops the value of the `team` field
///
/// [expand]: Expand
/// [expression]: Expr
/// [lookup]: Lookup
/// [table]: crate::Table
pub type Context = Record;

/// May be used as part of an [`Expand`] [expression] to provide a new [`Context`]
/// from a [`Record`] in another table.
///
/// The current [`Context`] need not be the [`Record`] that is currently being expanded.
/// Nested lookups are explicitly supported, each one using the [`Record`]
/// returned by the previous lookup as the current [`Context`] to find the next [`Context`].
///
/// # Examples
///
/// ```bash
/// {{country.team}}
/// # ^^^^^^^ this is the lookup
///
/// {{country@country.team}}
/// # ^^^^^^^^^^^^^^^ this is the same lookup with an explicit index
///
/// {{country@Enemy Country.team}}
/// # ^^^^^^^^^^^^^^^^^^^^^ this lookup will index the `country` table using the value of `Enemy Country`
///
/// {{country.team.team@Enemy.code}}
/// # ^^^^^^^^^^^^^^^^^^^^^^^ this is a nested lookup:
/// // first country is looked up, then team is looked up, then team is looked up again using the `Enemy` field in team
/// ```
///
/// [expression]: Expr
#[derive(Clone, Debug, PartialEq)]
pub struct Lookup {
    /// The field used to index the current [`Context`] for a [table index] to perform this lookup.
    ///
    /// if `index` is None, `table_name` is used to index the current [`Context`].
    /// ```bash
    /// # indexes the current Context with "country", then uses that value to index the table "team"
    /// {{country.team}}
    ///
    /// # indexes the current Context with "Enemy Country", then uses that value to index the table "team"
    /// {{country@Enemy Country.team}}
    /// ```
    ///
    /// [table index]: crate::Table::index
    pub index: Option<String>,

    /// The name of the [table] to lookup.
    ///
    /// A Record in this [table] will be found and the current [`Context`] will be set to that [`Record`]
    ///
    /// [table]: crate::Table
    pub table_name: String,
}

impl Lookup {
    pub fn direct(table: &str) -> Self {
        Lookup {
            table_name: table.into(),
            index: None,
        }
    }

    pub fn indirect(table: &str, index: &str) -> Self {
        Lookup {
            table_name: table.into(),
            index: Some(index.into()),
        }
    }

    pub fn run<'c, 'b>(
        &self,
        context: &'c Context,
        defs: &'c HashMap<String, Table>,
        block_contexts: &'b InheritedContext,
    ) -> anyhow::Result<&'c Context>
    where
        'b: 'c,
    {
        let index = self.index.as_ref().unwrap_or(&self.table_name);

        if let Some(block_ctx) = block_contexts.get(index) {
            return Ok(block_ctx);
        }

        let key = context.get(index).ok_or_else(|| {
            anyhow!("Failed lookup: field `{index}` did not exist in context `{context:?}`")
        })?;

        let table = defs
            .get(&self.table_name)
            .ok_or_else(|| anyhow!("Failed lookup: no table named `{}`", &self.table_name))?;

        let context = table.index(key)?.ok_or_else(|| {
            anyhow!(
                "Failed lookup: expected to find a {} with $id={}",
                &self.table_name,
                &key
            )
        })?;

        Ok(context)
    }
}

/// This [expression] expands into a value when the template is populated.
///
/// It's currently the only supported [expression] in [popvars] and is certainly the most common.
///
/// > note: The following examples will use the tables found in the full sc-mod example.
///
/// # Examples
///
/// A "direct expansion" expands to the value of `country`
///
/// ```bash
/// {{country}}
/// ```
/// ```text
/// Germany
/// ```
///
///
/// A "lookup expansion" expands to the value of "team" for the country.
///
/// See ['Lookup'] for more detail on lookups
///
/// ```bash
/// {{country.team}}
/// ```
/// ```text
/// Allies
/// ```
///
///
/// [expression]: Expr
/// [popvars]: crate
#[derive(Clone, Debug, PartialEq)]
pub struct Expand {
    /// lookup.get(field)
    pub field: String,

    /// for lookup in path { context = defs.get(context.get(lookup.index.unwrap_or(lookup.table_name)).get() }
    pub path: Vec<Lookup>,
}

impl Expand {
    pub fn new(field: &str) -> Self {
        Expand {
            field: field.into(),
            path: Vec::new(),
        }
    }

    pub fn with_lookup(field: &str, path: Lookup) -> Self {
        Expand {
            field: field.into(),
            path: vec![path],
        }
    }

    pub fn with_nested_lookups(field: &str, path: Vec<Lookup>) -> Self {
        Expand {
            field: field.into(),
            path,
        }
    }

    pub fn run(
        &self,
        record: &Record,
        defs: &HashMap<String, Table>,
        context: &InheritedContext,
    ) -> anyhow::Result<String> {
        let mut current_context: &Record = record;

        for lookup in &self.path {
            current_context = lookup.run(current_context, defs, context)?;
        }

        let value = current_context.get(&self.field).ok_or_else(|| {
            anyhow!(
                "Failed expansion: context is missing field `{}`",
                &self.field,
            )
        })?;

        Ok(value.clone())
    }
}

impl FromStr for Expr {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use parsely::*;

        let (expr, _) = expr.then_skip(end()).parse(s).own_err()?;
        Ok(expr)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Comparison {
    expand: Expand,
    comparator: Comparator,
    value: Value,
}

impl Comparison {
    pub fn new(expand: Expand, comparator: Comparator, value: Value) -> Self {
        Comparison {
            expand,
            comparator,
            value,
        }
    }

    pub fn matches(
        &self,
        record: &Record,
        def: &HashMap<String, Table>,
        ctx: &InheritedContext,
    ) -> anyhow::Result<bool> {
        let value = self
            .expand
            .run(record, def, ctx)
            .with_context(|| format!("Failed expansion during comparison: `{:?}`", &self.expand))?;

        let matches = match &self.value {
            Value::Int(where_value) => self.comparator.compare(
                &value
                    .parse::<i64>()
                    .context(format!("Expected `{value}` field to be a signed integer"))?,
                where_value,
            ),
            Value::Uint(where_value) => self.comparator.compare(
                &value.parse::<u64>().context(format!(
                    "Expected `{value}` field to be an unsigned integer"
                ))?,
                where_value,
            ),
            Value::Float(where_value) => self.comparator.compare(
                &value
                    .parse::<f64>()
                    .context(format!("Expected `{value}` field to be a float"))?,
                where_value,
            ),
            Value::Text(where_value) => self.comparator.compare(&value, where_value),
        };

        Ok(matches)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Uint(u64),
    Float(f64),
    Text(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Comparator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

impl Comparator {
    pub fn compare<T: PartialEq + PartialOrd>(&self, a: T, b: T) -> bool {
        match self {
            Comparator::Equal => a == b,
            Comparator::NotEqual => a != b,
            Comparator::GreaterThan => a > b,
            Comparator::LessThan => a < b,
            Comparator::GreaterThanOrEqual => a >= b,
            Comparator::LessThanOrEqual => a <= b,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BlockExpr {
    ForTag(ForTag),
    If(Comparison),
}

impl BlockExpr {
    /// return the name used to close this [`BlockExpr`]
    fn close(&self) -> &'static str {
        match self {
            BlockExpr::ForTag(_) => "for",
            BlockExpr::If(_) => "if",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForTag {
    pub new_context_name: String,
    lookup: Lookup,
    where_clause: Option<Comparison>,
    pub other_clause: bool,
}

impl FromStr for ForTag {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (for_tag, _) = for_tag().parse(s).own_err()?;
        Ok(for_tag)
    }
}

impl ForTag {
    pub fn ctx_idx(&self) -> ContextIndex {
        let table_name = self.lookup.table_name.clone();
        let where_clause = self.where_clause.clone();
        let other_clause = self.other_clause;
        match (where_clause, other_clause) {
            (None, false) => ContextIndex::Table { table_name },
            (Some(where_clause), false) => ContextIndex::FilteredTableWhere {
                table_name,
                where_clause,
            },
            (None, true) => ContextIndex::FilteredTableOther {
                table_name,
                index: self.lookup.index.clone(),
            },
            (Some(where_clause), true) => ContextIndex::FilteredTableOtherWhere {
                table_name,
                where_clause,
                index: self.lookup.index.clone(),
            },
        }
    }
}
