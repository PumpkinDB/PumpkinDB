# UUINT[size]/EQUAL?

{% method %}

Compares two topmost items for equality as unsigned sized integers.

Input stack: `a b`

Output stack: `c`

`UUINT[size]/EQUAL?` will push `1` if they are equal, `0` otherwise.

{% common %}

```
PumpkinDB> 1u8 2u8 UUINT8/EQUAL?
0x00
```

{% endmethod %}

## Allocation

Runtime allocation for supporting primitives

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than two items on the stack

[InvalidValue](../errors/InvalidValue.md) If either the operands or the result cause an overflow, or if `a` or `b` are too big for `size`.

## Tests

```test
equal_u8 : u100u8 100u8 UUINT8/EQUAL?.
equal_diff_size_u8 : 100u8 100u8 UINT8/EQUAL? NOT.
requires_two_items_u8_0 : [UINT8/EQUAL?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_u8_1 : [1u8 UINT8/EQUAL?] TRY UNWRAP 0x04 EQUAL?.

equal_u16 : 100u16 100u16 UINT16/EQUAL?.
equal_diff_size_u16 : 100u16 100u16 UINT16/EQUAL? NOT.
requires_two_items_u16_0 : [UINT16/EQUAL?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_u16_1 : [1u16 UINT16/EQUAL?] TRY UNWRAP 0x04 EQUAL?.

equal_u32 : 100u32 100u32 UINT32/EQUAL?.
equal_diff_size_u32 : 100u32 100u32 UINT32/EQUAL? NOT.
requires_two_items_u32_0 : [UINT32/EQUAL?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_u32_1 : [1u32 UINT32/EQUAL?] TRY UNWRAP 0x04 EQUAL?.

equal_u64 : 100u64 100u64 UINT64/EQUAL?.
equal_diff_size_u64 : 100u64 100u64 UINT64/EQUAL? NOT.
requires_two_items_u64_0 : [UINT64/EQUAL?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_u64_1 : [1u64 UINT64/EQUAL?] TRY UNWRAP 0x04 EQUAL?.
```
