template:

```
`{{`index with spaces`}}={{outer_table@`index with spaces`.code}}` will not be included in the for loop due to the use of "other"
{@ for other outer in outer_table@`index with spaces` where code > 0 @}[{{outer.$id}}={{outer.code}}]{@ end for @}
```

output:

```
`a=100` will not be included in the for loop due to the use of "other"
[b=200][c=300]
```

vars:

```
foo,index with spaces
1,a
```

outer_table:

```
$id,code
a,100
b,200
c,300
```
