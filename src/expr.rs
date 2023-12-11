use std::{collections::HashMap, str::FromStr};

use crate::{Record, Table};

#[derive(Debug, PartialEq)]
pub enum Expr {
    Expand(Expand),
}
impl Expr {
    pub fn run(self, context: &Context, defs: &HashMap<String, Table>) -> anyhow::Result<String> {
        match self {
            Expr::Expand(expand) => expand.run(context, defs),
        }
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

    pub fn run<'c>(
        &self,
        context: &'c Context,
        defs: &'c HashMap<String, Table>,
    ) -> anyhow::Result<&'c Context> {
        let index = self.index.as_ref().unwrap_or(&self.table_name);
        let key = context.get(index).ok_or_else(|| {
            anyhow!("Failed lookup: field `{index}` did not exist in context `{context:?}`")
        })?;
        let table = defs
            .get(&self.table_name)
            .ok_or_else(|| anyhow!("Failed lookup: no def named `{}`", &self.table_name))?;
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
#[derive(Debug, PartialEq)]
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

    pub fn run(self, context: &Context, defs: &HashMap<String, Table>) -> anyhow::Result<String> {
        let mut current_context: &Context = context;

        for lookup in self.path {
            current_context = lookup.run(current_context, defs)?;
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

        let (expr, _) = expr.then_skip(end()).parse(s)?;
        Ok(expr)
    }
}

use anyhow::anyhow;
use parsing::expr;

mod parsing {
    use parsely::*;

    use super::*;

    pub fn expr(input: &str) -> ParseResult<Expr> {
        let (inner, remaining) = brackets("{{", "}}").lex(input)?;

        let (expand, _) = expand().parse(inner)?;

        let expr = Expr::Expand(expand);

        Ok((expr, remaining))
    }

    fn expand() -> impl Parse<Output = Expand> {
        // Maximum of 100 nested lookups... for sanity's sake!
        lookup
            .many(0..=100)
            .then(segment("${@.}"))
            .then_end()
            .map(|(path, field)| Expand { path, field })
    }

    fn lookup(input: &str) -> ParseResult<'_, Lookup> {
        let ((table_name, index), remaining) = segment("${@.}")
            .then(explicit_index().optional())
            .then_skip('.') // this unambigously means we have a segment remaining after a lookup (required for the field)
            .parse(input)?;

        Ok((Lookup { index, table_name }, remaining))
    }

    fn explicit_index() -> impl Parse<Output = String> {
        '@'.skip_then(segment("@."))
    }

    fn brackets(open: &'static str, close: &'static str) -> impl Lex {
        segment_lexer("}").pad_with(open, close)
    }

    fn segment_lexer(terminating_chars: &'static str) -> impl Lex {
        escape_lexer('\\').or(none_of(terminating_chars)).many(1..)
    }

    fn segment(terminating_chars: &'static str) -> impl Parse<Output = String> {
        let char = char_if(|c| !terminating_chars.contains(c) && c != '\\')
            .map(|s| s.chars().next().unwrap());

        let inner = escape('\\').or(char).many(1..);

        inner.map(|chars| chars.into_iter().collect::<String>())
    }

    fn escape(esc: char) -> impl Parse<Output = char> {
        esc.skip_then(switch([
            (esc, esc),
            ('{', '{'),
            ('}', '}'),
            ('$', '$'),
            ('.', '.'),
            ('@', '@'),
        ]))
    }

    fn escape_lexer(esc: char) -> impl Lex {
        esc.then(esc.or(one_of("{}$.@")))
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
            let (matched, _) = parser
                .parse(input)
                .unwrap_or_else(|_| panic!("expected: {expected_match:?} for input: {input}"));
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
        fn test_brackets() {
            assert_lex_match(brackets("{{", "}}"), "{{country}}", "country");

            assert_lex_fails(
                brackets("{{", "}}"),
                "{{country}",
                "missing closing bracket",
            );
            assert_lex_fails(
                brackets("{{", "}}"),
                "{country}}",
                "missing opening bracket",
            );
        }

        #[test]
        fn test_brackets_retain_escape_chars() {
            assert_lex_match(
                brackets("{{", "}}"),
                r"{{country@\{foo\}}}",
                r"country@\{foo\}",
            );
            assert_lex_match(brackets("{{", "}}"), "{{${{}}}}.)", "${{");
            assert_lex_match(brackets("{{", "}}"), "{{country}}", "country");
            assert_lex_match(brackets("{{", "}}"), r"{{cou\ntry}}", r"cou\ntry");

            assert_lex_fails(
                brackets("{{", "}}"),
                r"{{country\}}",
                "missing closing bracket",
            );
            assert_lex_fails(
                brackets("{{", "}}"),
                r"$\(country)",
                "missing opening bracket",
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
            assert_parse_match(explicit_index(), r"@\$foo", "$foo".into());
            assert_parse_match(explicit_index(), r"@\@foo", "@foo".into());
            assert_parse_match(explicit_index(), r"@\$foo\.bar", "$foo.bar".into());
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

            assert_parse_fails(lookup, "table", "missing trailing .");
            assert_parse_fails(lookup, "table@foo", "missing trailing .");
            assert_parse_fails(lookup, "table@foo@bar.", "@ used twice (no trailing .)");
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

            assert_parse_fails(
                expand(),
                "table@.field",
                "@ is followed by . which is invalid if not escaped",
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
                "{{country@Enemy Country.code}}",
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
                "{{country@Enemy Country.team.code}}",
                Expr::Expand(Expand::with_nested_lookups(
                    "code",
                    vec![
                        Lookup::indirect("country", "Enemy Country"),
                        Lookup::direct("team"),
                    ],
                )),
            );

            // seems like it ought to fail - but whitespace is valid and significant!
            assert_parse_match(
                expr,
                "{{awueif q34t@23r .r}}",
                Expr::Expand(Expand::with_lookup(
                    "r",
                    Lookup::indirect("awueif q34t", "23r "),
                )),
            );
        }
    }
}
