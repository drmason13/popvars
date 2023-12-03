# Definitions

definitions come in two kinds:

- `vars` - collection of _variables_ used to populate templates
- `types` - named sets of _variables_ that can be referenced from `vars` or other `types`

That's a bit abstract, so let's see an example.

Bob has a name (Bob), an age (42) and a job (builder).

Bob might be recorded in a spreadsheet like this:

| name | age | job     |
| ---- | --- | ------- |
| Bob  | 42  | builder |

`name`, `age` and `job` are the variables that Bob has. These variables belong to Bob. Bob is one set of variables. There may be other sets of variables, but this one is Bob.

The value of `age` for Bob is `42`.

We can add lots of other people, who will also have names, ages, and jobs. They have the same set of variables as Bob, but with their own values.

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

`types` aren't available to be interpolated directly, they have to be accessed via `vars`. For example we can't just put `$(sector)` in our template when we're populating our template, we have to get to it through `job`:

`$(name) works in $(job.sector)`

becomes

> Bob works in construction Alice works in legal Mo works in hair

When popvars sees `builder` in the `job` field, it knows that `job` is a `type`, so it looks for `builder` in `job` using the `$id` field and finds that the `sector` is `construction`.

---

How are these definitions made?

Definitions can be made using spreadsheets! Each spreadsheet should have **one** `vars` sheet, and as many other sheets that they like defining `types`. The name of each sheet (other than the `vars` sheet) is the name of the `type`.

The `vars` sheet is the one used to populate templates.

For every sheet:

- Each **column** in a sheet is a `field`.
- Each **row** in a sheet is an `instance`.

Here's a more complete example listing the names and contents of some sheets in a spreadsheet file (a.k.a. a workbook).

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

<`vars` sheet>

```
country,city,
Germany,Konigsberg
France,Marseille
UK,Gibraltar
USA,Boston
Soviet Union,Smolensk
```

To populate a template with variables using these definitions, we need a template! templates are just plain text files:

<`demo.txt` template file>

```
$(country) with code $(country.code) contains $(city).
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

You might have noticed that there's no Italy in the output. Not every defined type has to be used, only the ones referenced by `vars` are used.

`$outfile` is another special field like `$id` used to tell popvars where to put the output. Each row is populated and then added to the `$outfile`, in order. This lets you output to multiple different files at once!

Another way to specify multiple different output files at once is to make a template that, once populated with variables, is used to define the path to a file to write to. This is done using the `TBD` arg.

<`vars` sheet>

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
$(country) with code $(country.code) contains $(city).
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

Note: You might want to have a field that has a `.` in its name. In order to refer to that field, and not get an error about accessing a non-existent type, you will have to "escape" the `.` by prefixing it with a `\`, e.g. `$(fav\. color)` accesses a value named `fav. color`.

You may store additional type definitions in separate files so they can be easily shared between different templates. Load each file containing templates in using the `-t, --template` arg. e.g. `popvars -d "national morale.ods" -t "red alert types.ods" -t "geography.ods"`
