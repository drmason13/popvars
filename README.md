# Definitions

`definitions` are collections of _variables_.

Within popvars, there's one special definition to be aware of: `$vars`.

- `$vars` contains the variables used to populate your template.

All other `definitions` have user defined names, we call these _named definitions_, or `types` for short.

- `types` are named sets of variables that can be referenced from `definitions`

That's a bit abstract, so let's see an example.

Bob has a name (Bob), an age (42) and a job (builder).

Bob might be recorded as a single row in a spreadsheet like this:

| name | age | job     |
| ---- | --- | ------- |
| Bob  | 42  | builder |

- The row is a `variable`
- The columns `name`, `age` and `job` are the `fields`
- The table is a `definition`
- each cell (other than the header) contains a `value`

The `value` of `age` for the first `variable` in this `definition` is `42`.

All `variables` in this `definition` have the same `fields`, just like each row in a table has the same columns.

Each `variable` in the `definition` has its own `value` for each `field`.

We can put lots of other people to in our table along with Bob, who will also have names, ages, and jobs but with their own values.

| name  | age | job         |
| ----- | --- | ----------- |
| Bob   | 42  | builder     |
| Alice | 44  | lawyer      |
| Mo    | 57  | hairdresser |

Now we have a lot of people who have jobs, let's define a job `type` and associate some values with each job:

| $id         | avg. salary | sector       |
| ----------- | ----------- | ------------ |
| builder     | £42000      | construction |
| hairdresser | £29000      | hair         |
| lawyer      | £77000      | legal        |

`$id` is a special field that popvars understands. `$id` is what we use to refer to a job, so it must be different for every job we want to use.

`types` aren't available to be interpolated directly, they have to be accessed via `$vars`. For example we can't just put `{{sector}}` in our template, we have to get to it through `job`:

`{{name}} works in {{job.sector}}`

becomes

> Bob works in construction

> Alice works in legal

> Mo works in hair

When popvars sees `builder` in the `job` field, it knows that `job` is a `type`, so it looks for a variable in `job` with the `$id` of `builder` and finds one, in that `variable` the `sector` is `construction`.

---

How are these definitions made?

It's no coincidence that `definitions` look a lot like spreadsheets: Definitions can be made using spreadsheets!

Each spreadsheet should have **one** `$vars` sheet, and as many other sheets that they like defining `types`. The name of each sheet (other than the `$vars` sheet) is the name of the `type`.

The `$vars` sheet is the one used to populate templates.

For every sheet:

- Each **column** in a sheet is a `field`.
- Each **row** in a sheet is an `instance`.

Here's a more complete example listing the names and contents of some sheets in a spreadsheet file (a.k.a. a workbook).

<`$vars` sheet>

```
country,city,
Germany,Konigsberg
France,Marseille
UK,Gibraltar
USA,Boston
Soviet Union,Smolensk
```

<`country` sheet>

```
$id,code
Germany,45
France,40
UK,112
Italy,59
USA,115
Soviet Union,116
```

To populate a template with variables using these definitions, we need a template! templates are just plain text files:

<`demo.txt` template file>

```
{{country}} with code {{country.code}} contains {{city}}.
```

The above template populated with the above definitions will make the following output:

<`out.txt` output>

```
Germany with code 45 contains Konigsberg.
France with code 40 contains Marseille.
UK with code 112 contains Gibraltar.
USA with code 115 contains Boston.
Soviet Union with code 116 contains Smolensk.
```

You might have noticed that there's no Italy in the output. Not every defined type has to be used, only the ones referenced by `$vars` are used.

`$outfile` is another special field like `$id` used to tell popvars where to put the output. Each row is populated and then added to the `$outfile`, in order. This lets you output to multiple different files at once!

<`$vars` sheet>

```
$outfile,country,city,
germany.txt,Germany,Konigsberg
france.txt,France,Marseille
uk.txt,UK,Gibraltar
usa.txt,USA,Boston
ussr.txt,Soviet Union,Smolensk
```

To populate a template with variables using these definitions, we need a template! templates are just plain text files:

<`demo.txt` template file>

```
{{country}} with code {{country.code}} contains {{city}}.
```

The above template populated with the above definitions will make the following output:

<`germany.txt` output>

```
Germany with code 45 contains Konigsberg.
```

<`france.txt` output>

```
France with code 40 contains Marseille.
```

<`uk.txt` output>

```
UK with code 112 contains Gibraltar.
```

<`usa.txt` output>

```
USA with code 115 contains Boston.
```

<`ussr.txt` output>

```
Soviet Union with code 116 contains Smolensk.
```

Note: You might want to have a field that has a `.` in its name. In order to refer to that field, and not get an error about accessing a non-existent type, you will have to "escape" the `.` by prefixing it with a `\`, e.g. `{{fav\. color}}` accesses a value named `fav. color`.

You may store additional type definitions in separate files so they can be easily shared between different templates. Load each file containing templates in using the `-t, --template` arg. e.g. `popvars -d "national morale.ods" -t "red alert types.ods" -t "geography.ods"`

# Advanced usage

## Looping (done)

you can repeat parts of a template.

This is done using a block expression `{@ ... @}` with the following syntax:

```
{@ for allied_country in country where team = "Allies" @}<template to loop>{@ end for @}
```

Loops through each record in country that satisfies the "where clause", which refers to the field `team` with the context of each country record.

### Context while Looping (done)

Within the example loop, `allied_country` refers to the current Record in the country Table being templated inside the loop.

Previous contexts remain available, including previous loops!

Note: Loops that define a new context with the same name as an existing context **override** that context within the loop.

## Includes (not done yet)

To manage the complexity of authoring templates, popvars supports reusing templates inside other templates.

This is done using a block expression `{@ ... @}` with the following syntax:

```
{@ pop template_path @}
```

where `template_path` is the path to your template file relative to where you are running popvars.

e.g.

```
{@ pop macros/header.txt @}
```

Optionally, new fields may be added to the context for only that template:

```
{@ pop template_path with field as new_field @}
```

where `field` is a field in the current context whose value you want to use and `new_field` is the new field to provide within the current context while populating the template.

This allows you to effectively rename fields to match those in a re-usable template.

You can also provide new values directly in the block expression by placing them within double quotes:

```
{@ pop template_path with "value" as new_field @}
```

Escape double quote and backslash using backslash:

```
{@ pop template_path with "value containing \"double quotes\" and backslashes \\ too for good measure" as new_field @}
```
