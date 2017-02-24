# AND

{% method -%}

Compares two boolean values and push `1` if both are true, `0` otherwise.

Input stack: `a` `b`

Output stack: `c`

{% common -%}

Pretty much how you would expect boolean values to behave:
```
PumpkinDB> 1 1 AND
0x01
PumpkinDB> 0 1 AND
0x00
```

{% endmethod %}

## Allocation

None

## Errors

[InvalidValue](./errors/InvalidValue.md) error if the both values are not booleans.

## Tests

```test
true_and_true : 1 1 AND 1 EQUAL?.
true_and_false : 1 0 AND 0 EQUAL?.
false_and_false : 0 0 AND 0 EQUAL?.
and_bool_a : [2 0 AND] TRY UNWRAP 0x03 EQUAL?.
and_bool_b : [0 2 AND] TRY UNWRAP 0x03 EQUAL?.
```
