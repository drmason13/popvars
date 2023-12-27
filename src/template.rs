//! Contains types and methods to compile templates from strings into a kind of execution plan
//! to populate the template
//!
//! * read template file -> String
//! * parse -> Vec<Node>
//! * compile -> Template (with ContextIndexes set for each Expr)

use std::collections::HashMap;

use parsely::result_ext::*;

use crate::{
    expr::{template, Block, BlockExpr, Node, WhereClause},
    table::Value,
    Definition, Expr, Record,
};

/// [`Template`]s consists of an ordered list of [`Node`]s to be rendered in order and a
/// [`ContextIndex`] to access the required [Context] for populating
#[derive(PartialEq, Debug)]
pub struct Template {
    nodes: Vec<CompiledNode>,
}

#[derive(PartialEq, Debug)]
pub enum CompiledNode {
    Text(String),
    Expr(Expr, AdditionalContext),
    Block(CompiledBlock),
}

/// A [`CompiledBlock`] is a [`Block`] of [`CompiledNode`]s
#[derive(PartialEq, Debug)]
pub struct CompiledBlock {
    pub expr: BlockExpr,
    pub nodes: Vec<CompiledNode>,
}

impl Template {
    pub fn compile(input: &str) -> anyhow::Result<Self> {
        let (nodes, _) = template(input).own_err()?;

        let mut ctx: AdditionalContext = HashMap::new();

        let compiled_nodes: Vec<CompiledNode> = nodes
            .into_iter()
            .map(|node| Template::compile_node(node, &mut ctx))
            .collect();

        Ok(Template {
            nodes: compiled_nodes,
        })
    }

    pub fn compile_node(node: Node, ctx: &mut AdditionalContext) -> CompiledNode {
        match node {
            Node::Text(string) => CompiledNode::Text(string),
            Node::Expr(expr) => match expr {
                Expr::Expand(_) => CompiledNode::Expr(expr.clone(), ctx.clone()),
            },
            Node::Block(Block { expr, nodes }) => match expr {
                BlockExpr::ForIn(ref for_in) => {
                    ctx.insert(for_in.new_context_name.clone(), for_in.ctx_idx());
                    let compiled_block_nodes: Vec<CompiledNode> = nodes
                        .into_iter()
                        .map(|node| Template::compile_node(node, ctx))
                        .collect();

                    CompiledNode::Block(CompiledBlock {
                        expr: expr.clone(),
                        nodes: compiled_block_nodes,
                    })
                }
            },
        }
    }

    #[allow(unused)]
    pub fn pop(&self, record: &Record, def: &Definition) -> anyhow::Result<String> {
        let mut output = String::new();

        for node in self.nodes.iter() {
            node.pop(&mut output, record, def)?;
        }

        Ok(output)
    }
}

impl CompiledNode {
    pub fn pop(
        &self,
        output: &mut String,
        record: &Record,
        def: &Definition,
    ) -> anyhow::Result<()> {
        match self {
            CompiledNode::Expr(Expr::Expand(expand), ctx) => {
                output.push_str(&expand.run(record, def, ctx)?);
                Ok(())
            }
            CompiledNode::Block(block) => match &block.expr {
                BlockExpr::ForIn(_) => {
                    for node in &block.nodes {
                        node.pop(output, record, def)?;
                    }
                    Ok(())
                }
            },
            CompiledNode::Text(s) => {
                output.push_str(s);
                Ok(())
            }
        }
    }
}

pub type AdditionalContext = HashMap<String, ContextIndex>;

/// [`ContextIndex`] can be used to [`index()`] [`Definition`] to return &[Context] to use while populating a [`Template`]
///
/// [`index()`]: Definition::index
#[derive(PartialEq, Debug, Clone)]
pub enum ContextIndex {
    /// A single Value inline rather than an Index as such
    ///
    /// e.g. `{@ for n in [1, 2, 3] @}n is {{n}}{@ end for @}`
    Value(Value),

    /// A list of values inline, rather than an Index as such
    ValueList(Vec<Value>),

    /// Selects an entire Table to provide as context
    Table { table_name: String },

    /// A Where Clause selects a filtered list of Records from a Table to provide as context
    FilteredTable {
        table_name: String,
        where_clause: WhereClause,
    },
}
