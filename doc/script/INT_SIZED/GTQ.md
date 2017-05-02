# INT[size]/GT?

{% method -%}

Compares two topmost items as signed sized integers.

Input stack: `a b`

Output stack: `a`

`INT[size]/LT?` will push `1` if `a` is strictly bigger than `b`, `0` otherwise.

{% common -%}

```
PumpkinDB> 1 -1 INT8/GT?.
0x01
PumpkinDB> -8 -1 INT8/GT?.
0x00
```

{% endmethod %}

## Allocation

Runtime allocation for supporting primitives

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than two items on the stack

[InvalidValue](../errors/InvalidValue.md) error if `a` or `b` cannot be signed integers, or if `a` or `b` are too big for `size`.

## Tests

```test
less_i8 : +1i8 +2i8 INT8/GT? NOT.
greater_i8 : +2i8 +1i8 INT8/GT?.
equal_i8 : +1i8 +1i8 INT8/LT? NOT.
requires_two_items_i8_0 : [INT8/LGT?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_i8_1 : [1i8 INT8/GT?] TRY UNWRAP 0x04 EQUAL?.

less_i16 : +1i16 +2i16 INT16/GT? NOT.
greater_i16 : +2i16 +1i16 INT16/GT?.
equal_i16 : +1i16 +1i16 INT16/LT? NOT.
requires_two_items_i16_0 : [INT16/LGT?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_i16_1 : [1i16 INT16/GT?] TRY UNWRAP 0x04 EQUAL?.

less_i32 : +1i32 +2i32 INT32/GT? NOT.
greater_i32 : +2i32 +1i32 INT32/GT?.
equal_i32 : +1i32 +1i32 INT32/LT? NOT.
requires_two_items_i32_0 : [INT32/LGT?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_i32_1 : [1i32 INT32/GT?] TRY UNWRAP 0x04 EQUAL?.

less_i64 : +1i64 +2i64 INT64/GT? NOT.
greater_i64 : +2i64 +1i64 INT64/GT?.
equal_i64 : +1i64 +1i64 INT64/LT? NOT.
requires_two_items_i64_0 : [INT64/LGT?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_i64_1 : [1i64 INT64/GT?] TRY UNWRAP 0x04 EQUAL?.
```
