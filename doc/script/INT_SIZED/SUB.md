INT[size]/SUB
===

{% method -%}

Subtracts one signed sized integer from another, the size can be i8, i16, i32 or i64.

Input stack: `a` `b`

Output stack: `c`

`SUB` will subtract of `b` from `a` and push it to the top of the stack.

{% common -%}

```
PumpkinDB> +2i8 +1i8 INT8/SUB.
0x01
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
works_i8 : +2i8 +1i8 INT8/SUB +1i8 EQUAL?.
negative_value_i8 : +1i8 +2i8 INT8/SUB -1i8 EQUAL?.
empty_stack : [INT8/SUB] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : [+1i8 INT8/SUB] TRY UNWRAP 0x04 EQUAL?.

works_i16 : +2i16 +1i16 INT16/SUB +1i16 EQUAL?.
negative_value_i16 : +1i16 +2i16 INT16/SUB -1i16 EQUAL?.
empty_stack : [INT16/SUB] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : [+1i16 INT16/SUB] TRY UNWRAP 0x04 EQUAL?.

works_i32 : +2i32 +1i32 INT32/SUB +1i32 EQUAL?.
negative_value_i32 : +1i32 +2i32 INT32/SUB -1i32 EQUAL?.
empty_stack : [INT32/SUB] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : [+1i32 INT32/SUB] TRY UNWRAP 0x04 EQUAL?.

works_i64 : +2i64 +1i64 INT64/SUB +1i64 EQUAL?.
negative_value_i64 : +1i64 +2i64 INT64/SUB -1i64 EQUAL?.
empty_stack : [INT64/SUB] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : [+1i64 INT64/SUB] TRY UNWRAP 0x04 EQUAL?.
```
