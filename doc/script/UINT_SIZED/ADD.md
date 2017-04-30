UINT[size]/ADD
===

{% method -%}

Sums two unsigned sized integers, the size can be u8, u16, u32 or u64. 

Input stack: `a` `b`

Output stack: `c`

`AND` will push the sum of `a` and `b` to the top of the stack.

{% common -%}

```
PumpkinDB> 1u8 2u8 UINT8/ADD.
0x03
```

{% endmethod %}

## Allocation

Runtime allocations for decoding numbers and heap allocation
for the result.

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are less than two items on the stack

[InvalidValue](../errors/InvalidValue.md) error if `a` or `b` cannot be unsigned integers or if either the operands or the result cause an overflow.

## Tests

```test
works_u8 : 2u8 1u8 UINT8/ADD 3u8 EQUAL?.
empty_stack_u8 : [UINT8/ADD] TRY UNWRAP 0x04 EQUAL?.
empty_stack_u8_1 : [1u8 UINT8/ADD] TRY UNWRAP 0x04 EQUAL?.

works_u16 : 2u16 1u16 UINT16/ADD 3u16 EQUAL?.
empty_stack_u16 : [UINT16/ADD] TRY UNWRAP 0x04 EQUAL?.
empty_stack_u16_1 : [1u16 UINT16/ADD] TRY UNWRAP 0x04 EQUAL?.

works_u32 : 2u32 1u32 UINT32/ADD 3u32 EQUAL?.
empty_stack_u32 : [UINT32/ADD] TRY UNWRAP 0x04 EQUAL?.
empty_stack_u32_1 : [1u32 UINT32/ADD] TRY UNWRAP 0x04 EQUAL?.

works_u64 : 2u64 1u64 UINT64/ADD 3u64 EQUAL?.
empty_stack_u64 : [UINT64/ADD] TRY UNWRAP 0x04 EQUAL?.
empty_stack_u64_1 : [1u64 UINT64/ADD] TRY UNWRAP 0x04 EQUAL?.
```
