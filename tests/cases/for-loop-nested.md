template:

```
foo is in vars: {{foo}}

outer_table is in defs: {{outer_table.code}}
inner_table is in defs: {{inner_table.code}}
{@ for outer in outer_table @}
`outer.code` now refers to the same table as `outer_table.code`
{{outer.$id}}={{outer.code}}

about to go inside the inner loop:
{@ for inner in inner_table @}
`inner.code` now refers to the same table as `inner_table.code`
{{inner.$id}}={{inner.code}}

`outer.code` still refers to the same table as `outer_table.code` inside the inner loop
{{outer.$id}}={{outer.code}}
{@ end for @}

{@ end for @}
```

output:

```
foo is in vars: 1

outer_table is in defs: 100
inner_table is in defs: 111

`outer.code` now refers to the same table as `outer_table.code`
a=100

about to go inside the inner loop:

`inner.code` now refers to the same table as `inner_table.code`
aa=111

`outer.code` still refers to the same table as `outer_table.code` inside the inner loop
a=100

`inner.code` now refers to the same table as `inner_table.code`
bb=222

`outer.code` still refers to the same table as `outer_table.code` inside the inner loop
a=100



`outer.code` now refers to the same table as `outer_table.code`
b=200

about to go inside the inner loop:

`inner.code` now refers to the same table as `inner_table.code`
aa=111

`outer.code` still refers to the same table as `outer_table.code` inside the inner loop
b=200

`inner.code` now refers to the same table as `inner_table.code`
bb=222

`outer.code` still refers to the same table as `outer_table.code` inside the inner loop
b=200



```

vars:

```
foo,outer_table,inner_table
1,a,aa
```

outer_table:

```
$id,code
a,100
b,200
```

inner_table:

```
$id,code
aa,111
bb,222
```
