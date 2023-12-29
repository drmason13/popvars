template:

```
foo is in vars: {{foo}}

outer_table is in defs: {{outer_table.code}}
{@ for outer in outer_table @}
    `outer.code` now refers to the same table as `outer_table.code`
    {{outer.$id}}={{outer.code}}
{@ end for @}
```

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

## includes

includes/part one.txt:

```
This is another template that may be included in the topmost template. {{ included }}

It is free to include further includes inside itself: {@ pop `includes/part two.txt` with "Hi part two, it's 'part one' here" as `variable for part two` @}
```

includes/part two.txt:

```
{{variable for part two}}
```
