INT[size]/ADD
===

{% method -%}

Sums two signed sized integers, the size can be i8, i16, i32 or i64. 

Input stack: `a` `b`

Output stack: `c`

`AND` will push the sum of `a` and `b` to the top of the stack.

{% common -%}

```
PumpkinDB> +1i8 +2i8 INT8/ADD.
0x03
```

{% endmethod %}

## Allocation

Runtime allocations for decoding numbers and heap allocation
for the result.

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are less than two items on the stack

[InvalidValue](../errors/InvalidValue.md) error if `a` or `b` cannot be signed integers, or if either the operands or the result cause an overflow.

## Tests

```test
works_i8 : +2i8 +1i8 INT8/ADD +3i8 EQUAL?.
empty_stack_i8 : [INT8/ADD] TRY UNWRAP 0x04 EQUAL?.
empty_stack_i8_1 : [+1i8 INT8/ADD] TRY UNWRAP 0x04 EQUAL?.

works_i16 : +2i16 +1i16 INT16/ADD +3i16 EQUAL?.
empty_stack_i16 : [INT16/ADD] TRY UNWRAP 0x04 EQUAL?.
empty_stack_i16_1 : [+1i16 INT16/ADD] TRY UNWRAP 0x04 EQUAL?.

works_i32 : +2i32 +1i32 INT32/ADD +3i32 EQUAL?.
empty_stack_i32 : [INT32/ADD] TRY UNWRAP 0x04 EQUAL?.
empty_stack_i32_1 : [+1i32 INT32/ADD] TRY UNWRAP 0x04 EQUAL?.

works_i64 : +2i64 +1i64 INT64/ADD +3i64 EQUAL?.
empty_stack_i64 : [INT64/ADD] TRY UNWRAP 0x04 EQUAL?.
empty_stack_i64_1 : [+1i64 INT64/ADD] TRY UNWRAP 0x04 EQUAL?.
```
