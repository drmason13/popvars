template:

```
{{country}}{@ if `country`.`team`="Axis" @} (Axis){@ end if @}
```

output:

```
France
Germany (Axis)
Italy (Axis)
```

vars:

```
country
France
Germany
Italy
```

country:

```
$id,team
France,Allies
Germany,Axis
Italy,Axis
```
