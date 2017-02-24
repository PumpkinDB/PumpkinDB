# NOT

{% method -%}

Negates a boolean value.

Input stack: `a`

Output stack: `c`

`NOT` will push `1` if `a` is `0` and `0` if `a` is `1`.

{% common -%}

```
PumpkinDB> 0x01 NOT
0
PumpkinDB> 0x00 NOT
1
```

{% endmethod %}

## Allocation

None

## Errors

[InvalidValue](./errors/InvalidValue.md) error if the value being negated is not a boolean.

[EmptyStack](./errors/EmptyStack.md) error if there are less than one item on the stack

## Tests

```test
works : 1 NOT 0 EQUAL?.
works_1 : 0 NOT 1 EQUAL?.
invalid_value : [5 NOT] TRY UNWRAP 0x03 EQUAL?.
empty_stack : [NOT] TRY UNWRAP 0x04 EQUAL?.
```
