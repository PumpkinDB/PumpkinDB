UINT[size]/SUB
===

{% method -%}

Subtracts one unsigned integer from another, the size can be u8, u16, u32 or u64. 

Input stack: `a` `b`

Output stack: `c`

`SUB` will subtract of `b` from `a` and push it to the top of the stack.

{% common -%}

```
PumpkinDB> 2u8 1u8 UINT8/SUB.
0x01
```

{% endmethod %}

## Allocation

Runtime allocations for decoding numbers and heap allocation
for the result.

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are less than two items on the stack

[InvalidValue](../errors/InvalidValue.md) error if `a` is less than `b`

## Tests

```test
works_u8 : 2u8 1u8 UINT8/SUB 1u8 EQUAL?.
invalid_value_u8 : [1u8 2u8 UINT8/SUB] TRY UNWRAP 0x03 EQUAL?.
empty_stack_u8 : [UINT8/SUB] TRY UNWRAP 0x04 EQUAL?.
empty_stack_u8_1 : [1u8 UINT8/SUB] TRY UNWRAP 0x04 EQUAL?.

works_u16 : 2u16 1u16 UINT16/SUB 1u16 EQUAL?.
invalid_value_u16 : [1u16 2u16 UINT16/SUB] TRY UNWRAP 0x03 EQUAL?.
empty_stack_u16 : [UINT16/SUB] TRY UNWRAP 0x04 EQUAL?.
empty_stack_u16_1 : [1u16 UINT16/SUB] TRY UNWRAP 0x04 EQUAL?.

works_u32 : 2u32 1u32 UINT32/SUB 1u32 EQUAL?.
invalid_value_u32 : [1u32 2u32 UINT32/SUB] TRY UNWRAP 0x03 EQUAL?.
empty_stack_u32 : [UINT32/SUB] TRY UNWRAP 0x04 EQUAL?.
empty_stack_u32_1 : [1u32 UINT32/SUB] TRY UNWRAP 0x04 EQUAL?.

works_u64 : 2u64 1u64 UINT64/SUB 1u64 EQUAL?.
invalid_value_u64 : [1u64 2u64 UINT64/SUB] TRY UNWRAP 0x03 EQUAL?.
empty_stack_u64 : [UINT64/SUB] TRY UNWRAP 0x04 EQUAL?.
empty_stack_u64_1 : [1u64 UINT64/SUB] TRY UNWRAP 0x04 EQUAL?.
```
