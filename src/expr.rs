use std::{
    borrow::{Borrow, Cow},
    collections::HashMap,
    str::FromStr,
};

use crate::{Record, Table};

#[derive(Debug, PartialEq)]
pub enum Expr {
    Expand(Expand),
    Block(Block),
}

impl Expr {
    pub fn run(self, context: &Context, defs: &HashMap<String, Table>) -> anyhow::Result<String> {
        match self {
            Expr::Expand(expand) => expand.run(context, defs),
            // Expr::ForIn(for_in) => for_in.run(context, defs),
            _ => unimplemented!("Block Expressions not ready yet"),
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

        let (expr, _) = expr.then_skip(end()).parse(s).own_err()?;
        Ok(expr)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhereClause {
    field: String,
    comparator: Comparator,
    value: Value,
}

impl WhereClause {
    pub fn new(field: &str, comparator: Comparator, value: Value) -> Self {
        WhereClause {
            field: field.to_owned(),
            comparator,
            value,
        }
    }

    pub fn matches(&self, record: &Record, table_name: &str) -> anyhow::Result<bool> {
        let value = record.get(&self.field).ok_or_else(|| {
            anyhow!(
                "Failed expansion: context is missing field `{}`",
                &self.field,
            )
        })?;

        let matches = match &self.value {
            Value::Int(where_value) => self.comparator.compare(
                where_value,
                &value.parse::<i64>().context(format!(
                    "Expected `{}` field in `{}` table to be a signed integer",
                    self.field, table_name
                ))?,
            ),
            Value::Uint(where_value) => self.comparator.compare(
                where_value,
                &value.parse::<u64>().context(format!(
                    "Expected `{}` field in `{}` table to be a unsigned integer",
                    self.field, table_name
                ))?,
            ),
            Value::Float(where_value) => self.comparator.compare(
                where_value,
                &value.parse::<f64>().context(format!(
                    "Expected `{}` field in `{}` table to be a float",
                    self.field, table_name
                ))?,
            ),
            Value::Text(where_value) => self.comparator.compare(where_value, value),
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

#[derive(Debug, PartialEq)]
pub struct Block {
    pub tag: BlockTag,
    pub content: String,
}

#[derive(Debug, PartialEq)]
pub enum BlockTag {
    ForIn(ForIn),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForIn {
    new_context_name: String,
    table_name: String,
    where_clause: Option<WhereClause>,
}

impl ForIn {
    pub fn run<'d>(
        self,
        _context: &Context,
        defs: &'d HashMap<String, Table>,
    ) -> anyhow::Result<Table> {
        let new_context = defs.get(&self.new_context_name).ok_or_else(|| {
            anyhow!(
                "Failed expansion: defs is missing table `{}`",
                &self.new_context_name,
            )
        })?;

        let new_context = new_context
            .iter()
            .map(|record| {
                if let Some(wc) = &self.where_clause {
                    Ok((record, wc.matches(record, self.table_name.as_str())?))
                } else {
                    Ok((record, true))
                }
            })
            .filter(|result| match result {
                Ok((_, keep)) => *keep,
                Err(_) => true,
            })
            .map(|result| result.map(|(record, _)| Cow::Borrowed(record.borrow())))
            .collect::<anyhow::Result<Vec<_>>>()?;

        // Ok(Table::new(self.new_context_name, new_context))
        todo!()
    }
}

impl FromStr for ForIn {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (for_in, _) = for_in().parse(s).own_err()?;
        Ok(for_in)
    }
}

use anyhow::{anyhow, Context as _};
use parsely::{result_ext::*, Parse};
use parsing::{expr, for_in};

mod parsing {
    use parsely::*;

    use super::*;

    pub fn expr(input: &str) -> ParseResult<Expr> {
        let error = match outer_brackets("{{", "}}").lex(input).offset(input) {
            Ok((inner, remaining)) => match expand().parse(inner).offset(input) {
                Ok((expand, _)) => {
                    let expr = Expr::Expand(expand);
                    return Ok((expr, remaining));
                }
                Err(e) => e,
            },
            Err(e) => e,
        };

        let (block_expr, remaining) = block_expr.parse(input).offset(input).merge(error)?;

        Ok((block_expr, remaining))
    }

    // {@ ___ ... @}  <content>  {@ end ___ @}
    //    ^^^  these must be the same   ^^^
    pub fn block_expr(input: &str) -> ParseResult<Expr> {
        let (opening_tag, remaining) = outer_brackets("{@", "@}").lex(input).offset(input)?;

        let closing_tag = if "for".pad().lex(opening_tag).is_ok() {
            // we'll use this to lex the closing tag - it has to match the opening tag
            "for"
        } else {
            // unrecognised block expression... one day I'll be add a label like this to the error
            return Err(parsely::Error::no_match(input));
        };

        match closing_tag {
            "for" => {
                let (tag, _) = for_in().parse(opening_tag).offset(input).offset(input)?;
                let (block, remaining) = until("{@ end for").lex(remaining).offset(input)?;
                todo!()
                // How do we parse nested {@ for ... @} ... {@ end for @}
                // **inside** a {@ for ... @} ... {@ end for @} ??
                // oh we just find either the close {@ end for @} OR another opening tag, in which case we
                // will have to include it in this block as a part of the String.
                // increment a depth counter, keep going until we get back to 0 +1 for each open, -1 for each close
                // we'll either get back to - or fail to match input at which point we just bail!

                // we already need a way to push new Contexts somewhere we can use them to expand expressions

                // this is all complicated enough to warrant a new function/parser/combinator :)
            }
            _ => unreachable!("unexpected tag name in block expression - this is a bug!"),
        }
    }

    // for `allied_country` in `country` where team="Allies"
    pub fn for_in() -> impl Parse<Output = ForIn> {
        "for"
            .pad()
            .skip_then("`".skip_then(segment("`")))
            .then_skip("in".pad())
            .then("`".skip_then(segment("`")))
            .then(where_clause.pad().optional())
            .map(|((ctx, table), where_clause)| ForIn {
                new_context_name: ctx,
                table_name: table,
                where_clause,
            })
    }

    fn value() -> impl Parse<Output = Value> {
        let string = "\"".skip_then(segment("\""));

        string
            .map(Value::Text)
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
            .skip_then(segment("<!=>"))
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

    fn outer_brackets(open: &'static str, close: &'static str) -> impl Lex {
        segment_lexer(close).pad_with(open, close)
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
            let (matched, _) = dbg!(parser.parse(input)).unwrap_or_else(|_| {
                panic!("parser failed to match: {expected_match:?} for input: {input}")
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
            assert_lex_match(outer_brackets("{{", "}}"), r"{{cou\ntry}}", r"cou\ntry");

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
}
