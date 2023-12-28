template:

```
foo is in vars: {{foo}}

{@ pop macros/header.txt with outer_table as header @}
```

output:

```
foo is in vars: 1

Header One Hundred (100)
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
Header {{header.title}} ({{header.code}})
```
