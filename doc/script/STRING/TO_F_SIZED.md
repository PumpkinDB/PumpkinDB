# STRING/->F[size]

{% method -%}

Convert a string to sized floating point number. Size can be f32 or f64.

Input stack: `numeric string`

Output stack: `number`

{% common -%}

```
PumpkinDB> "1.24" STRING/->FLOAT32.
0xbf9eb852
```

{% endmethod %}

## Allocation

Space for floating point number.

## Errors

[EmptyStack](../errors/EmptyStack.md) error if stack is empty.

[InvalidValue](../errors/InvalidValue.md) error if stack value cannot be converted to integer.

## Tests

```test
works : "4.2" STRING/->F32 4.2f32 EQUAL?.
neg_works : "-4.2" STRING/->F32 -4.2f32 EQUAL?.
empty_stack : [STRING/->F32] TRY UNWRAP 0x04 EQUAL?.
invalid_value : ["NOT A NUM" STRING/->F32] TRY UNWRAP 0x03 EQUAL?.
```
