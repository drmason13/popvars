use popvars::Definition;

pub struct TestCase {
    pub definition: Definition,
    pub template: String,
    pub expected: String,
    pub includes: Vec<(String, String)>,
}

impl TestCase {
    pub fn run(self) -> anyhow::Result<()> {
        let actual = popvars::pop(self.template.as_str(), self.definition)?;
        let first = actual.join("");

        assert_eq!(self.expected, first);
        Ok(())
    }
}
pub use parsing::test_case as parse_test_case;

mod parsing {
    use super::*;
    use indoc::indoc;
    use parsely::{none_of, result_ext::*, until, ws, Lex, Parse, ParseResult};

    pub fn code_block() -> impl Parse<Output = String> {
        // note: code_block will contain a trailing \n
        "```"
            .then(ws())
            .skip_then(until("```").map(str::to_string))
            .then_skip("```")
    }

    pub fn named_code_block(name: &'static str) -> impl Parse<Output = String> {
        name.then(":").pad().skip_then(code_block())
    }

    pub fn dynamic_named_code_block() -> impl Parse<Output = (String, String)> {
        none_of("\r\n:")
            .many(1..)
            .map(str::to_string)
            .then_skip(":")
            .pad()
            .then(code_block())
    }

    pub fn test_case(input: &str) -> ParseResult<TestCase> {
        let (((template, expected), vars), remaining) = named_code_block("template")
            .then(named_code_block("output"))
            .then(named_code_block("vars"))
            .parse(input)
            .offset(input)?;

        let (defs, remaining) = dynamic_named_code_block()
            .many(0..20)
            .or_until("## includes")
            .parse(remaining)
            .offset(input)?;

        let (_, remaining) = "## includes"
            .pad()
            .optional()
            .lex(remaining)
            .offset(input)?;

        let (includes, remaining) = dynamic_named_code_block()
            .many(0..20)
            .parse(remaining)
            .offset(input)?;

        let definition = Definition::from_csv_strings(vars, defs.iter()).fail_conversion(input)?;

        let output = TestCase {
            definition,
            template,
            expected,
            includes,
        };

        Ok((output, remaining))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_case_works() -> Result<(), Box<dyn std::error::Error>> {
            let input = include_str!("test-parsing-test.md").replace("\r\n", "\n");

            let (output, _) = test_case.parse(input.as_str()).own_err()?;

            assert_eq!(
                output.template,
                String::from(indoc! {"
                foo is in vars: {{foo}}

                outer_table is in defs: {{outer_table.code}}
                {@ for outer in outer_table @}
                    `outer.code` now refers to the same table as `outer_table.code`
                    {{outer.$id}}={{outer.code}}
                {@ end for @}
            "})
            );

            assert_eq!(
                output.expected,
                String::from(indoc! {"
                foo is in vars: 1

                outer_table is in defs: 100
                
                    `outer.code` now refers to the same table as `outer_table.code`
                    a=100
                
                    `outer.code` now refers to the same table as `outer_table.code`
                    b=200
                
                    `outer.code` now refers to the same table as `outer_table.code`
                    c=300

            "})
            );

            assert_eq!(output.definition.defs.len(), 1);
            assert_eq!(
                output
                    .definition
                    .defs
                    .get("outer_table")
                    .unwrap()
                    .records
                    .len(),
                3
            );
            assert_eq!(
                output.definition.defs.get("outer_table").unwrap().records[1]
                    .get("code")
                    .unwrap(),
                "200"
            );
            assert_eq!(output.definition.vars.records.len(), 1);

            assert_eq!(output.includes.len(), 2);
            assert_eq!(output.includes[0], ("includes/part one.txt".into(), "This is another template that may be included in the topmost template. {{ included }}\n\nIt is free to include further includes inside itself: {@ pop `includes/part two.txt` with \"Hi part two, it's 'part one' here\" as `variable for part two` @}\n".into()));

            Ok(())
        }

        #[test]
        fn named_code_block_works() -> Result<(), Box<dyn std::error::Error>> {
            let input = include_str!("test-parsing-test.md").replace("\r\n", "\n");
            let (matched, remaining) = named_code_block("template")
                .parse(input.as_str())
                .own_err()?;
            assert_eq!(
                matched,
                indoc! {"
                    foo is in vars: {{foo}}

                    outer_table is in defs: {{outer_table.code}}
                    {@ for outer in outer_table @}
                        `outer.code` now refers to the same table as `outer_table.code`
                        {{outer.$id}}={{outer.code}}
                    {@ end for @}
                "}
            );
            assert!(remaining.starts_with(indoc! {"


                        output:

                        ```
                        foo is in vars: 1

                        outer_table is in defs: 100

                            `outer.code` now refers to the same table as `outer_table.code`
                            a=100

                            `outer.code` now refers to the same table as `outer_table.code`
                            b=200

                            `outer.code` now refers to the same table as `outer_table.code`
                            c=300

                        ```

                        vars:

                        ```
                        foo,outer_table
                        1,a
                        ```

                        outer_table:

                        ```
                        $id,code
                        a,100
                        b,200
                        c,300
                        ```
                    "}));
            Ok(())
        }
    }
}
