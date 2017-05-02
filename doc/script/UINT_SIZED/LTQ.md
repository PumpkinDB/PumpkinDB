# UINT[size]/LT?

{% method -%}

Compares two topmost items as unsigned sized integers.

Input stack: `a b`

Output stack: `a`

`UINT[size]/LT?` will push `1` if `a` is strictly smaller than `b`, `0` otherwise.

{% common -%}

```
PumpkinDB> 1u8 2u8 UINT8/LT?.
1
PumpkinDB> 2u8 1u8 UINT8/LT?.
0
```

{% endmethod %}

## Allocation

Runtime allocation for supporting primitives

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are less than two items on the stack
[InvalidValue](../errors/InvalidValue.md) error if `a` or `b` cannot be unsigned integers or if `a` or `b` are too big for `size`.

## Tests

```test
less_u8 : 1u8 2u8 UINT8/LT?.
greater_u8 : 2u8 1u8 UINT8/GT? NOT.
equal_u8 : 1u8 1u8 UINT8/LT?  NOT.
requires_two_items_u8_0 : [UINT8/LT?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_u8_1 : [1u8 UINT8/LT?] TRY UNWRAP 0x04 EQUAL?.

less_u16 : 1u16 2u16 UINT16/LT?.
greater_u16 : 2u16 1u16 UINT16/GT? NOT.
equal_u16 : 1u16 1u16 UINT16/LT?  NOT.
requires_two_items_u16_0 : [UINT16/LT?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_u16_1 : [1u16 UINT16/LT?] TRY UNWRAP 0x04 EQUAL?.

less_u32 : 1u32 2u32 UINT32/LT?.
greater_u32 : 2u32 1u32 UINT32/GT? NOT.
equal_u32 : 1u32 1u32 UINT32/LT?  NOT.
requires_two_items_u32_0 : [UINT32/LT?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_u32_1 : [1u32 UINT32/LT?] TRY UNWRAP 0x04 EQUAL?.

less_u64 : 1u64 2u64 UINT64/LT?.
greater_u64 : 2u64 1u64 UINT64/GT? NOT.
equal_u64 : 1u64 1u64 UINT64/LT?  NOT.
requires_two_items_u64_0 : [UINT64/LT?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_u64_1 : [1u64 UINT64/LT?] TRY UNWRAP 0x04 EQUAL?.
```
