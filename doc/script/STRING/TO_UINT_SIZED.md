# STRING/->UINT[size]

{% method -%}

Convert a string to unsigned sized integer. Size can be u8, u16, u32, or u64.

Input stack: `numeric string`

Output stack: `number`

{% common -%}

```
PumpkinDB> "42" STRING/->UINT32.
0x0000002a
```

{% endmethod %}

## Allocation

Space for integer.

## Errors

[EmptyStack](../errors/EmptyStack.md) error if stack is empty.

[InvalidValue](../errors/InvalidValue.md) error if stack value cannot be converted to integer.

## Tests

```test
works : "4" STRING/->UINT32 4u32 EQUAL?.
empty_stack : [STRING/->UINT32] TRY UNWRAP 0x04 EQUAL?.
invalid_value : ["NOT A NUM" STRING/->UINT32] TRY UNWRAP 0x03 EQUAL?.
```
