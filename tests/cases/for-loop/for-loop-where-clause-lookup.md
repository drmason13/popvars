template:

```
foo is in vars: {{foo}}

outer_table is in defs: {{outer_table.code}}
Note that the $id in this for loop where clause refers to *the $id of outer_table* This is pretty confusing I expect!
{@ for outer in outer_table where bar_table@code.bar = "bar3" @}
    {{outer.$id}}={{outer.code}}
{@ end for @}
```

output:

```
foo is in vars: 1

outer_table is in defs: 100
Note that the $id in this for loop where clause refers to *the $id of outer_table* This is pretty confusing I expect!

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

bar_table:

```
$id,bar
100,bar1
200,bar2
300,bar3
```
