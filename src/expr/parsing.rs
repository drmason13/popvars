use parsely::*;

use super::*;

pub fn template(input: &str) -> ParseResult<Vec<Node>> {
    node().many(1..).then_end().parse(input).offset(input)
}

pub fn node() -> impl Parse<Output = Node> {
    (block.map(Node::Block))
        .or(expr.map(Node::Expr))
        .or(text().map(Node::Text))
}

// Some plain text between expressions
pub fn text() -> impl Parse<Output = String> {
    content_escape()
        .many(1..)
        .or_until("{@".or("{{"))
        .collect::<String>()
}

pub fn outer_brackets(open: &'static str, close: &'static str) -> impl Lex {
    let inner = expr_escape().lexing().many(1..).or_until(close);
    (inner).pad_with(open, close)
}

pub fn expr(input: &str) -> ParseResult<Expr> {
    let (inner, remaining) = outer_brackets("{{", "}}").lex(input)?;
    let (expr, close) = expr_escape()
        .lexing()
        .many(1..)
        .or_until("}}")
        .lex(inner)
        .offset(input)?;

    let _ = end().lex(close)?;

    let (expand, after) = expand().pad().parse(expr).offset(input)?;
    let _ = end().lex(after)?;

    let expr = Expr::Expand(expand);
    Ok((expr, remaining))
}

// {@ ___ ... @}  <content>  {@ end ___ @}
//    ^^^  these must be the same   ^^^
pub fn block(input: &str) -> ParseResult<Block> {
    let (expr, remaining) = block_expr(input)?;
    let tag = expr.close();

    let (nodes, remaining) = node()
        .many(..)
        .or_until(close_block_expr(tag))
        .then_skip(close_block_expr(tag))
        .parse(remaining)?;

    Ok((Block { expr, nodes }, remaining))
}

pub fn close_block_expr(name: &str) -> impl Lex + '_ {
    "{@".then(ws().optional())
        .then("end ")
        .then(token(name))
        .then(ws().optional())
        .then("@}")
}

// {@ ___ ... @}
pub fn block_expr(input: &str) -> ParseResult<BlockExpr> {
    let (_, content) = "{@".then(ws().optional()).lex(input)?;

    let (tag, _) = until(" ").lex(content)?;

    match tag {
        "for" => for_in().map(BlockExpr::ForIn),
        _ => return Err(parsely::Error::no_match(content).offset(input)),
    }
    .then_skip(ws().optional().then("@}"))
    .parse(content)
    .offset(input)
}

// for `allied_country` in `country` where team="Allies"
pub fn for_in() -> impl Parse<Output = ForIn> {
    "for"
        .pad()
        .skip_then(other_clause)
        .then(segment(expr_escape(), " ").then_skip(" "))
        .then_skip("in".pad())
        .then(lookup)
        .then(where_clause.pad().optional())
        .map(|(((other_clause, ctx), lookup), where_clause)| ForIn {
            new_context_name: ctx,
            lookup,
            other_clause,
            where_clause,
        })
}

fn other_clause(input: &str) -> ParseResult<'_, bool> {
    if let Ok((_, remaining)) = "other ".lex(input) {
        Ok((true, remaining))
    } else {
        Ok((false, input))
    }
}

fn value() -> impl Parse<Output = Value> {
    (string('"').map(Value::Text))
        .or(uint::<u64>().map(Value::Uint))
        .or(int::<i64>().map(Value::Int))
        .or(float::<f64>().map(Value::Float))
}

// where team="Allies"
fn where_clause(input: &str) -> ParseResult<WhereClause> {
    let cmp = switch([
        ("!=", Comparator::NotEqual),
        (">=", Comparator::GreaterThanOrEqual),
        ("<=", Comparator::LessThanOrEqual),
        (">", Comparator::GreaterThan),
        ("<", Comparator::LessThan),
        ("=", Comparator::Equal),
    ]);

    let (((field, comparator), value), remaining) = "where"
        .pad()
        .skip_then(segment(expr_escape(), "<!=>"))
        .then(cmp)
        .then(value())
        .parse(input)
        .offset(input)?;

    let where_clause = WhereClause {
        field,
        comparator,
        value,
    };

    Ok((where_clause, remaining))
}

fn expand() -> impl Parse<Output = Expand> {
    // Maximum of 100 nested lookups... for sanity's sake!
    lookup
        .then_skip('.') // this unambigously means we have a segment remaining after a lookup (required for the field)
        .many(1..=100)
        .then(segment(expr_escape(), "{@ .}"))
        .then_skip(ws().many(..))
        .map(|(path, field)| Expand { path, field })
        .or(segment(expr_escape(), "{@.}")
            .then_skip(ws().many(..))
            .map(|field| Expand {
                path: Vec::new(),
                field,
            }))
}

fn lookup(input: &str) -> ParseResult<'_, Lookup> {
    let ((table_name, index), remaining) = segment(expr_escape(), "{@ .}")
        .then(explicit_index().optional())
        .parse(input)?;

    Ok((Lookup { index, table_name }, remaining))
}

fn explicit_index() -> impl Parse<Output = String> {
    '@'.skip_then(segment(expr_escape(), "@ ."))
}

fn string(quote: char) -> impl Parse<Output = String> {
    quote.skip_then(
        str_escape(quote)
            .many(1..)
            .or_until(quote)
            .collect::<String>()
            .then_skip(quote),
    )
}

fn segment(
    escape: impl Parse<Output = char>,
    terminating_chars: &'static str,
) -> impl Parse<Output = String> {
    string('`').or(escape
        .many(1..)
        .or_until(one_of(terminating_chars))
        .collect::<String>())
}

fn expr_escape() -> EscapeSequence<5, Parsing> {
    parsely::escape(
        '\\',
        [
            ('\\', '\\'), //
            ('@', '@'),
            ('{', '{'),
            ('}', '}'),
            ('.', '.'),
        ],
    )
}

fn content_escape() -> EscapeSequence<4, Parsing> {
    parsely::escape(
        '\\',
        [
            ('\\', '\\'), //
            ('@', '@'),
            ('{', '{'),
            ('}', '}'),
        ],
    )
}

fn str_escape(quote: char) -> EscapeSequence<6, Parsing> {
    parsely::escape(
        '\\',
        [
            ('\\', '\\'), //
            ('@', '@'),
            ('{', '{'),
            ('}', '}'),
            ('.', '.'),
            (quote, quote),
        ],
    )
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use super::*;

    fn assert_lex_match(lexer: impl Lex, input: &str, expected_match: &str) {
        let (matched, _) = lexer
            .lex(input)
            .unwrap_or_else(|_| panic!("expected: {expected_match} for input: {input}"));
        assert_eq!(matched, expected_match, "for input: {input}");
    }

    fn assert_lex_fails(lexer: impl Lex, input: &str, reason: &str) {
        let result = lexer.lex(input);
        assert!(
            result.is_err(),
            r#"expected lex to fail due to: "{reason}" for input: {input}"#
        );
    }

    fn assert_parse_match<O: PartialEq + Debug>(
        parser: impl Parse<Output = O>,
        input: &str,
        expected_match: O,
    ) {
        let (matched, _) = parser.parse(input).unwrap_or_else(|e| {
            panic!(
                "parser failed to match: {expected_match:?} for input: {input} with error: {e:?}"
            )
        });
        assert_eq!(matched, expected_match, "for input: {input}");
    }

    fn assert_parse_fails(parser: impl Parse, input: &str, reason: &str) {
        let result = parser.parse(input);
        assert!(
            result.is_err(),
            r#"expected parse to fail due to: "{reason}" for input: {input}"#
        );
    }

    #[test]
    fn test_block_text() {
        let block = "{@ for `field` in `table_name` @}loop content{@ end for @}";
        let (nodes, remaining) = template(block).unwrap();

        assert_eq!(remaining, "");
        assert_eq!(
            nodes[0],
            Node::Block(Block {
                expr: BlockExpr::ForIn(ForIn {
                    new_context_name: "field".into(),
                    lookup: Lookup::direct("table_name"),
                    where_clause: None,
                    other_clause: false,
                }),
                nodes: vec![Node::Text("loop content".into())],
            })
        );
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn test_block_without_backticks() {
        let block = "{@ for field in table_name @}loop content{@ end for @}";
        let (nodes, remaining) = template(block).unwrap();

        assert_eq!(remaining, "");
        assert_eq!(
            nodes[0],
            Node::Block(Block {
                expr: BlockExpr::ForIn(ForIn {
                    new_context_name: "field".into(),
                    lookup: Lookup::direct("table_name"),
                    where_clause: None,
                    other_clause: false,
                }),
                nodes: vec![Node::Text("loop content".into())],
            })
        );
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn test_block_expr() {
        let block = "{@ for `field` in `table_name` @}{{loop expr}}{@ end for @}";
        let (nodes, remaining) = template(block).unwrap();

        assert_eq!(remaining, "");
        assert_eq!(
            nodes[0],
            Node::Block(Block {
                expr: BlockExpr::ForIn(ForIn {
                    new_context_name: "field".into(),
                    lookup: Lookup::direct("table_name"),
                    where_clause: None,
                    other_clause: false,
                }),
                nodes: vec![Node::Expr(Expr::Expand(Expand {
                    field: "loop expr".into(),
                    path: vec![]
                }))],
            })
        );
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn test_block() {
        let block = indoc::indoc! {"
            text content.{{expr}}
            {@ for `field` in `table_name` @}loop content{{loop expr}}{@ end for @}
            text content after loop
        "};
        let (nodes, remaining) = template(block).unwrap();

        assert_eq!(remaining, "");
        assert_eq!(nodes[0], Node::from_text("text content."));
        assert_eq!(nodes[1], Node::Expr(Expr::from_str("{{expr}}").unwrap()));
        assert_eq!(nodes[2], Node::from_text("\n"));
        assert_eq!(
            nodes[3],
            Node::Block(Block {
                expr: BlockExpr::ForIn(ForIn {
                    new_context_name: "field".to_string(),
                    lookup: Lookup::direct("table_name"),
                    where_clause: None,
                    other_clause: false,
                }),
                nodes: vec![
                    Node::from_text("loop content"),
                    Node::Expr(Expr::Expand(Expand {
                        field: "loop expr".to_string(),
                        path: vec![],
                    }))
                ]
            })
        );
        assert_eq!(nodes[4], Node::from_text("\ntext content after loop\n"));
        assert_eq!(nodes.len(), 5);
    }

    #[test]
    fn test_for_other() {
        let block = "{@ for other `field` in `table_name` @}inner{@ end for @}";
        let (nodes, remaining) = template(block).unwrap();

        assert_eq!(remaining, "");
        assert_eq!(
            nodes[0],
            Node::Block(Block {
                expr: BlockExpr::ForIn(ForIn {
                    new_context_name: "field".into(),
                    lookup: Lookup::direct("table_name"),
                    where_clause: None,
                    other_clause: true,
                }),
                nodes: vec![Node::from_text("inner")],
            })
        );
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn test_for_loop() {
        let block = indoc::indoc! {"
            foo is in vars: {{foo}}

            outer_table is in defs: {{outer_table.code}}
            {@ for outer in outer_table @}
                `outer.code` now refers to the same table as `outer_table.code`
                {{outer.$id}}={{outer.code}}
            {@ end for @}
        "};
        let (nodes, remaining) = template(block).unwrap();

        assert_eq!(remaining, "");
        assert_eq!(nodes[0], Node::from_text("foo is in vars: "));
        assert_eq!(nodes[1], Node::Expr(Expr::from_str("{{foo}}").unwrap()));
        assert_eq!(nodes[2], Node::from_text("\n\nouter_table is in defs: "));
        assert_eq!(
            nodes[3],
            Node::Expr(Expr::from_str("{{outer_table.code}}").unwrap())
        );
        assert_eq!(nodes[4], Node::from_text("\n"));
        assert_eq!(
            nodes[5],
            Node::Block(Block {
                expr: BlockExpr::ForIn(ForIn {
                    new_context_name: "outer".to_string(),
                    lookup: Lookup::direct("outer_table"),
                    where_clause: None,
                    other_clause: false,
                }),
                nodes: vec![
                    Node::from_text("\n    `outer.code` now refers to the same table as `outer_table.code`\n    "),
                    Node::Expr(Expr::from_str("{{outer.$id}}").unwrap()),
                    Node::from_text("="),
                    Node::Expr(Expr::from_str("{{outer.code}}").unwrap()),
                    Node::from_text("\n"),
                ]
            })
        );
        assert_eq!(nodes[6], Node::from_text("\n"));
    }

    #[test]
    fn test_outer_brackets() {
        assert_lex_match(outer_brackets("{{", "}}"), "{{country}}", "country");

        assert_lex_fails(
            outer_brackets("{{", "}}"),
            "{{country}",
            "missing closing bracket",
        );
        assert_lex_fails(
            outer_brackets("{{", "}}"),
            "{country}}",
            "missing opening bracket",
        );

        assert_lex_match(outer_brackets("{@", "@}"), "{@country@}", "country");

        assert_lex_fails(
            outer_brackets("{@", "@}"),
            "{@country}",
            "missing closing bracket",
        );
        assert_lex_fails(
            outer_brackets("{@", "@}"),
            "{country@}",
            "missing opening bracket",
        );
    }

    #[test]
    fn test_brackets_retain_escape_chars() {
        assert_lex_match(
            outer_brackets("{{", "}}"),
            r"{{country@\{foo\}}}",
            r"country@\{foo\}",
        );
        assert_lex_match(outer_brackets("{{", "}}"), "{{${{}}}}.)", "${{");
        assert_lex_match(outer_brackets("{{", "}}"), "{{country}}", "country");
        assert_lex_match(outer_brackets("{{", "}}"), r"{{me\@myself}}", r"me\@myself");

        assert_lex_fails(
            outer_brackets("{{", "}}"),
            r"{{cou\ntry}}",
            "invalid escape sequence (\\ needs escaping)",
        );
        assert_lex_fails(
            outer_brackets("{{", "}}"),
            r"{{country\}}",
            "missing closing bracket",
        );
        assert_lex_fails(
            outer_brackets("{{", "}}"),
            r"$\(country)",
            "missing opening bracket",
        );
    }

    #[test]
    fn test_node_with_remainder() {
        assert_parse_match(
            node(),
            "{{loop expr}}{@ end for @}",
            Node::Expr(Expr::Expand(Expand {
                field: "loop expr".into(),
                path: vec![],
            })),
        );
    }

    #[test]
    fn test_close_block_expr() {
        assert_lex_match(
            close_block_expr("for"),
            expr.parse("{{loop expr}}{@ end for @}").unwrap().1,
            "{@ end for @}",
        );
    }

    #[test]
    fn test_explicit_index() {
        assert_parse_match(explicit_index(), "@foo", "foo".into());
        assert_parse_match(explicit_index(), "@foo@bar", "foo".into());

        assert_parse_fails(explicit_index(), "No @", "missing leading @");
    }

    #[test]
    fn test_explicit_index_removes_escape_chars() {
        assert_parse_match(explicit_index(), r"@\@foo", "@foo".into());
        assert_parse_match(explicit_index(), r"@\@foo\.bar", "@foo.bar".into());
        assert_parse_match(explicit_index(), r"@foo\{bar\}", "foo{bar}".into());
    }

    #[test]
    fn test_lookup() {
        assert_parse_match(
            lookup,
            "table@index.field",
            Lookup::indirect("table", "index"),
        );

        assert_parse_match(lookup, "table.field", Lookup::direct("table"));
        assert_parse_match(lookup, "table", Lookup::direct("table"));
        assert_parse_match(lookup, "table@foo", Lookup::indirect("table", "foo"));
        assert_parse_match(lookup, "table@foo@bar.", Lookup::indirect("table", "foo"));
    }

    #[test]
    fn test_expand_direct() {
        assert_parse_match(expand(), "field", Expand::new("field"));
        assert_parse_match(expand(), r"\{field\}", Expand::new("{field}"));

        assert_parse_fails(expand(), "{{field}}", "{} are special chars, need escaping");
    }

    #[test]
    fn test_expand_lookups() {
        assert_parse_match(
            expand(),
            "table.field",
            Expand::with_lookup("field", Lookup::direct("table")),
        );

        assert_parse_match(
            expand(),
            "table@index.field",
            Expand::with_lookup("field", Lookup::indirect("table", "index")),
        );

        assert_parse_match(
            expand(),
            "`table name`.field",
            Expand::with_lookup("field", Lookup::direct("table name")),
        );

        assert_parse_match(
            expand(),
            "`table name`.`field name`",
            Expand::with_lookup("field name", Lookup::direct("table name")),
        );

        assert_parse_match(
            expand(),
            "table.`field name`",
            Expand::with_lookup("field name", Lookup::direct("table")),
        );

        assert_parse_match(
            expand(),
            "table@index.b.field",
            Expand::with_nested_lookups(
                "field",
                vec![Lookup::indirect("table", "index"), Lookup::direct("b")],
            ),
        );

        assert_parse_match(
            expand(),
            "a.b.c.d",
            Expand::with_nested_lookups(
                "d",
                vec![
                    Lookup::direct("a"),
                    Lookup::direct("b"),
                    Lookup::direct("c"),
                ],
            ),
        );
    }

    #[test]
    fn test_expr() {
        assert_parse_match(expr, "{{country}}", Expr::Expand(Expand::new("country")));
        assert_parse_match(
            expr,
            "{{country.code}}",
            Expr::Expand(Expand::with_lookup("code", Lookup::direct("country"))),
        );
        assert_parse_match(
            expr,
            "{{country@`Enemy Country`.code}}",
            Expr::Expand(Expand::with_lookup(
                "code",
                Lookup::indirect("country", "Enemy Country"),
            )),
        );
        assert_parse_match(
            expr,
            "{{country.team.code}}",
            Expr::Expand(Expand::with_nested_lookups(
                "code",
                vec![Lookup::direct("country"), Lookup::direct("team")],
            )),
        );
        assert_parse_match(
            expr,
            "{{country@`Enemy Country`.team.code}}",
            Expr::Expand(Expand::with_nested_lookups(
                "code",
                vec![
                    Lookup::indirect("country", "Enemy Country"),
                    Lookup::direct("team"),
                ],
            )),
        );

        assert_parse_match(
            expr,
            "{{ country@`Enemy Country`.team.code }}",
            Expr::Expand(Expand::with_nested_lookups(
                "code",
                vec![
                    Lookup::indirect("country", "Enemy Country"),
                    Lookup::direct("team"),
                ],
            )),
        );

        // seems like it ought to fail - and it does now!
        assert_parse_fails(
            expr,
            "{{awueif q34t@23r .r}}",
            "Whitespace is only significant if wrapped in `backticks`",
        );

        assert_parse_match(
            segment(expr_escape(), "@ ."),
            r"\.field",
            ".field".to_string(),
        );

        assert_parse_fails(
            expr,
            "{{table@.field}}",
            "@ is followed by . which is invalid if not escaped",
        );

        assert_parse_match(
            expr,
            r"{{table@dr\.index.code}}",
            Expr::Expand(Expand::with_lookup(
                "code",
                Lookup::indirect("table", "dr.index"),
            )),
        );
    }

    #[test]
    fn test_where_clause() {
        assert_parse_match(
            where_clause,
            r#"where team<="Allies""#,
            WhereClause::new(
                "team",
                Comparator::LessThanOrEqual,
                Value::Text("Allies".into()),
            ),
        );
    }
}
