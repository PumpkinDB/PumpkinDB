# OR

Compares two booleans values and return a `1` if either is true.

Input stack: `a` `b`
Output stack: `c`

`OR` will push `1` if `a` or `b` is `1`, otherwise it will push `0`.

## Allocation

None

## Errors

InvalidValue error if the both values are not booleans.

## Examples

```
0x01 0x01 OR => 1
0x00 0x01 OR => 1
0x00 0x00 OR => 0
```

## Tests

```
0x01 0x01 OR => 1
0x00 0x01 OR => 1
0x00 0x00 OR => 0
```