template:

```
foo is in vars: {{foo}}
outer_table is in defs: {{outer_table.code}}
```

output:

```
foo is in vars: 1
outer_table is in defs: 100
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
