# STRING/->INT[size]

{% method -%}

Convert a string to sized integer. Size can be i8, i16, i32, or i64.

Input stack: `numeric string`

Output stack: `number`

{% common -%}

```
PumpkinDB> "14" STRING/->INT32.
0x8000000e

```

{% endmethod %}

## Allocation

Space for integer.

## Errors

[EmptyStack](../errors/EmptyStack.md) error if stack is empty.

[InvalidValue](../errors/InvalidValue.md) error if stack value cannot be converted to integer.

## Tests

```test
works : "4" STRING/->INT32 4i32 EQUAL?.
neg_works : "-4" STRING/->INT32 -4i32 EQUAL?.
empty_stack : [STRING/->INT32] TRY UNWRAP 0x04 EQUAL?.
invalid_value : ["NOT A NUM" STRING/->INT32] TRY UNWRAP 0x03 EQUAL?.
```
