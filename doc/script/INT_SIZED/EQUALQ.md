# INT[size]/EQUAL?

{% method -%}

Compares two topmost items for equality as signed sized integers.

Input stack: `a b`

Output stack: `c`

`INT[size]/EQUAL?` will push `1` if they are equal, `0` otherwise.

{% common -%}

```
PumpkinDB> +1i8 +2i8 INT8/EQUAL?
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
equal_i8 : +100i8 +100i8 INT8/EQUAL?.
equal_diff_size_i8 : +100i8 -100i8 INT8/EQUAL? NOT.
requires_two_items_i8_0 : [INT8/EQUAL?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_i8_1 : [1i8 INT8/EQUAL?] TRY UNWRAP 0x04 EQUAL?.

equal_i16 : +100i16 +100i16 INT16/EQUAL?.
equal_diff_size_i16 : +100i16 -100i16 INT16/EQUAL? NOT.
requires_two_items_i16_0 : [INT16/EQUAL?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_i16_1 : [1i16 INT16/EQUAL?] TRY UNWRAP 0x04 EQUAL?.

equal_i32 : +100i32 +100i32 INT32/EQUAL?.
equal_diff_size_i32 : +100i32 -100i32 INT32/EQUAL? NOT.
requires_two_items_i32_0 : [INT32/EQUAL?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_i32_1 : [1i32 INT32/EQUAL?] TRY UNWRAP 0x04 EQUAL?.

equal_i64 : +100i64 +100i64 INT64/EQUAL?.
equal_diff_size_i64 : +100i64 -100i64 INT64/EQUAL? NOT.
requires_two_items_i64_0 : [INT64/EQUAL?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_i64_1 : [1i64 INT64/EQUAL?] TRY UNWRAP 0x04 EQUAL?.
```
