template:

```
foo is in vars: {{foo}}

outer_table is in defs: {{outer_table.code}}
{@ for outer in outer_table where code<=200 @}
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
