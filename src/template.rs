//! Contains types and methods to compile templates from strings into a kind of execution plan
//! to populate the template
//!
//! * read template file -> String
//! * parse -> Vec<Node>
//! * compile -> Template (with ContextIndexes set for each Expr)

use std::collections::HashMap;

use anyhow::anyhow;
use parsely::result_ext::*;

use crate::{
    expr::{template, Block, BlockExpr, Comparison, Node},
    table::Value,
    Definition, Expr, Record,
};

/// [`Template`]s consists of an ordered list of [`Node`]s to be rendered in order and a
/// [`ContextIndex`] of blocks to access the required context for populating
#[derive(PartialEq, Debug)]
pub struct Template {
    nodes: Vec<CompiledNode>,
}

#[derive(PartialEq, Debug)]
pub enum CompiledNode {
    Text(String),
    Expr(Expr),
    Block(CompiledBlock),
}

/// A [`CompiledBlock`] is a [`Block`] of [`CompiledNode`]s
#[derive(PartialEq, Debug)]
pub struct CompiledBlock {
    pub expr: BlockExpr,
    pub nodes: Vec<CompiledNode>,
    // not all Blocks provide new context
    pub block_ctx_idx: Option<(String, ContextIndex)>,
}

impl Template {
    pub fn compile(input: &str) -> anyhow::Result<Self> {
        let (nodes, _) = template(input).own_err()?;

        let compiled_nodes: Vec<CompiledNode> =
            nodes.into_iter().map(Template::compile_node).collect();

        Ok(Template {
            nodes: compiled_nodes,
        })
    }

    pub fn compile_node(node: Node) -> CompiledNode {
        match node {
            Node::Text(string) => CompiledNode::Text(string),
            Node::Expr(expr) => match expr {
                Expr::Expand(_) => CompiledNode::Expr(expr),
            },
            Node::Block(Block { expr, nodes }) => {
                let (compiled_block_nodes, block_ctx_idx) = match expr {
                    BlockExpr::ForTag(ref for_tag) => (
                        nodes
                            .into_iter()
                            .map(Template::compile_node)
                            .collect::<Vec<_>>(),
                        Some((for_tag.new_context_name.clone(), for_tag.ctx_idx())),
                    ),
                    BlockExpr::If(_) => (
                        nodes
                            .into_iter()
                            .map(Template::compile_node)
                            .collect::<Vec<_>>(),
                        None,
                    ),
                    BlockExpr::Include((template_path, field, include_var_name)) => {
                        dbg!(template_path, field, include_var_name);
                        unimplemented!("Includes are not yet implemented")
                    }
                };

                CompiledNode::Block(CompiledBlock {
                    expr,
                    nodes: compiled_block_nodes,
                    block_ctx_idx,
                })
            }
        }
    }

    #[allow(unused)]
    pub fn pop(&self, record: &Record, def: &Definition) -> anyhow::Result<String> {
        let mut output = String::new();

        let mut ctx: InheritedContext = HashMap::new();

        for node in self.nodes.iter() {
            node.pop(&mut output, record, def, &ctx)?;
        }

        Ok(output)
    }
}

impl CompiledNode {
    pub fn pop<'d, 'b>(
        &self,
        output: &mut String,
        record: &Record,
        def: &'d Definition,
        // blocks and like parent blocks
        ctx: &'b InheritedContext,
    ) -> anyhow::Result<()>
    where
        'b: 'd,
    {
        match self {
            CompiledNode::Expr(Expr::Expand(expand)) => {
                output.push_str(&expand.run(record, &def.defs, ctx)?);
                Ok(())
            }
            CompiledNode::Block(block) => match &block.expr {
                BlockExpr::ForTag(_) => {
                    let (ctx_name, ctx_idx) = &block
                        .block_ctx_idx
                        .as_ref()
                        .expect("ForTag always has a new Context");
                    let contexts = def
                        .index(ctx_idx, record, ctx)
                        .ok_or_else(|| anyhow!("Failed to index context `{ctx_name}` for block"))?;

                    // PERF: avoid clone?
                    for loop_ctx in contexts {
                        let mut merged_ctx = ctx.clone();
                        merged_ctx.insert(ctx_name.clone(), loop_ctx.clone());

                        for node in &block.nodes {
                            node.pop(output, record, def, &merged_ctx)?;
                        }
                    }
                    Ok(())
                }
                BlockExpr::If(comparison) => {
                    if comparison.matches(record, &def.defs, ctx)? {
                        for node in &block.nodes {
                            node.pop(output, record, def, ctx)?;
                        }
                    }

                    Ok(())
                }
                BlockExpr::Include((template_path, field, include_var_name)) => {
                    dbg!(template_path, field, include_var_name);
                    unimplemented!("Includes are not yet implemented")
                }
            },
            CompiledNode::Text(s) => {
                output.push_str(s);
                Ok(())
            }
        }
    }
}

pub type InheritedContext<'a> = HashMap<String, Record>;

/// [`ContextIndex`] can be used to [`index()`] [`Definition`] to return &[Context] to use while populating a [`Template`]
///
/// [`index()`]: Definition::index
#[derive(PartialEq, Debug, Clone)]
pub enum ContextIndex {
    /// A list of values inline, rather than an Index as such
    ///
    /// e.g. `{@ for n in [1, 2, 3] @}n is {{n}}{@ end for @}`
    ValueList(Vec<Value>),

    /// Selects an entire Table to provide as context
    Table { table_name: String },

    /// A Where Clause selects a filtered list of Records from a Table to provide as context
    FilteredTableWhere {
        table_name: String,
        where_clause: Comparison,
    },

    /// An Other Clause filters out one Record that matches a lookup from the given table Table, and selects all the others to provide as context
    FilteredTableOther {
        table_name: String,
        index: Option<String>,
    },

    /// A Where Clause and an Other Clause combined selects all other records that also match the Where Clause
    FilteredTableOtherWhere {
        table_name: String,
        where_clause: Comparison,
        index: Option<String>,
    },
}
