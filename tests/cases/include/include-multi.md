template:

```
{@ pop macros/header.txt with "The title" as title @}
{{foo}} is the content
{@ pop macros/footer.txt with bar as title @}
```

output:

```
Header The title
Hello World is the content
Footer Goodbye World
```

vars:

```
foo,bar
Hello World,Goodbye World
```

## includes

macros/header.txt:

```
Header {{title}}
```

macros/footer.txt:

```
Footer {{title}}
```
