# OR

{% method -%}

Compares two booleans values and return a `1` if either is true.

Input stack: `a` `b`
Output stack: `c`

`OR` will push `1` if `a` or `b` is `1`, otherwise it will push `0`.

{% common -%}

```
PumpkinDB> 0x01 0x01 OR
1
PumpkinDB> 0x00 0x01 OR
1
PumpkinDB> 0x00 0x00 OR
0
```

{% endmethod %}

## Allocation

None

## Errors

[InvalidValue](./errors/InvalidValue.md) error if the both values are not booleans.

[EmptyStack](./errors/EmptyStack.md) error if there are less than two items on the stack

## Tests

```test
invalid_value_1 : [5 1 OR] TRY UNWRAP 0x03 EQUAL?.
invalid_value_2 : [1 5 OR] TRY UNWRAP 0x03 EQUAL?.
requires_two_items_0 : [OR] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_1 : [1 OR] TRY UNWRAP 0x04 EQUAL?.
```
