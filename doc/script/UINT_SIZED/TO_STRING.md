# UINT[size]/->STRING

{% method -%}

Convert sized integer to string, the size can be u8, u16, u32 or u64.

Input stack: `number`

Output stack: `string-of-number`

`UINT{8,16,32,64}->STRING` pushes a string representation of given number to the top of the stack.

{% common -%}

```
PumpkinDB> 1024u32 UINT32/->STRING.
"1024"
```

{% endmethod %}

## Allocation

Space for string representation of number will be allocated.

## Errors

[EmptyStack](../errors/EmptyStack.md) error if stack is empty.

[InvalidValue](../errors/InvalidValue.md) error if stack value cannot be converted to integer.

## Tests

```test
works_u8 : 2u8 UINT8/->STRING "2" EQUAL?.
empty_stack : [UINT8/->STRING] TRY UNWRAP 0x04 EQUAL?.

works_u16 : 2u16 UINT16/->STRING "2" EQUAL?.
empty_stack : [UINT16/->STRING] TRY UNWRAP 0x04 EQUAL?.

works_u32 : 2u32 UINT32/->STRING "2" EQUAL?.
empty_stack : [UINT32/->STRING] TRY UNWRAP 0x04 EQUAL?.

works_u64 : 2u64 UINT64/->STRING "2" EQUAL?.
empty_stack : [UINT64/->STRING] TRY UNWRAP 0x04 EQUAL?.
```
