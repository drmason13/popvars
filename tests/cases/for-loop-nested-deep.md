template:

```
The innermost loop \{\@ for outer in outer_table \@\} replaces the context with the same in the outermost loop, so we see exactly the same output twice: once for each row in outer_table.
{@ for outer in outer_table @}{@ for inner in inner_table @}{@ for outer in outer_table @}{{inner.$id}}={{inner.code}},{{outer.$id}}={{outer.code}};{@ end for @}{@ end for @}{@ end for @}
```

output:

```
The innermost loop {@ for outer in outer_table @} replaces the context with the same in the outermost loop, so we see exactly the same output twice: once for each row in outer_table.
aa=111,a=100;aa=111,b=200;bb=222,a=100;bb=222,b=200;aa=111,a=100;aa=111,b=200;bb=222,a=100;bb=222,b=200;
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
