template:

```
foo is in vars: {{foo}}

{@ pop macros/header.txt with "The title" as title @}
```

output:

```
foo is in vars: 1

Header The title
```

vars:

```
foo,outer_table
1,a
```

## includes

macros/header.txt:

```
Header {{title}}
```
