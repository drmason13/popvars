template:

```
{@ for x in outer_table @}{@ pop macros/header.txt with x as header @}{@ end for @}
```

output:

```
[Header One Hundred][Header Two Hundred][Header Three Hundred]
```

vars:

```
foo,outer_table
1,a
```

outer_table:

```
$id,code,title
a,100,"One Hundred"
b,200,"Two Hundred"
c,300,"Three Hundred"
```

## includes

macros/header.txt:

```
[Header {{header}}]
```
