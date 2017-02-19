# AND

Compares two booleans values and return a `1` if both are true.

Input stack: `a` `b`
Output stack: `c`

`AND` will push `1` if `a` is `1` and `b` is `1`, otherwise it will push `0`.

## Allocation

None

## Errors

[InvalidValue](./ERRORS/InvalidValue.md) error if the both values are not booleans.

## Examples

```
0x01 0x01 AND => 1
0x00 0x01 AND => 0
```

## Tests

```test
true_and_true : 1 1 AND 1 EQUAL?.
true_and_false : 1 0 AND 0 EQUAL?.
false_and_false : 0 0 AND 0 EQUAL?.
and_bool_a : [2 0 AND] TRY UNWRAP 0x03 EQUAL?.
and_bool_b : [0 2 AND] TRY UNWRAP 0x03 EQUAL?.
```