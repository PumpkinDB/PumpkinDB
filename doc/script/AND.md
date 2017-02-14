# AND

Compares two booleans values and return a `1` if both are true.

Input stack: `a` `b`
Output stack: `c`

`AND` will push `1` if `a` is `1` and `b` is `1`, otherwise it will push `0`.

## Allocation

None

## Errors

InvalidValue error if the both values are not booleans.

## Examples

```
0x01 0x01 AND => 1
0x00 0x01 AND => 0
```

## Tests

```
0x01 0x01 AND => 1
0x00 0x01 AND => 0
```