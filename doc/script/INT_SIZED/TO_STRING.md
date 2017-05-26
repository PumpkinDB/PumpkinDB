# INT[size]/->STRING

{% method -%}

Convert sized integer to string, the size can be i8, i16, i32 or i64.

Input stack: `number`

Output stack: `string-of-number`

`INT{8,16,32,64}->STRING` pushes a string representation of given number to the top of the stack.

{% common -%}

```
PumpkinDB> 1024i32 INT32/->STRING.
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
works_i8 : 2i8 INT8/->STRING "2" EQUAL?.
neg_works_i8 : -2i8 INT8/->STRING "-2" EQUAL?.
empty_stack_i8 : [INT8/->STRING] TRY UNWRAP 0x04 EQUAL?.

works_i16 : 2i16 INT16/->STRING "2" EQUAL?.
neg_works_i16 : -2i16 INT16/->STRING "-2" EQUAL?.
empty_stack_i16 : [INT16/->STRING] TRY UNWRAP 0x04 EQUAL?.

works_i32 : 2i32 INT32/->STRING "2" EQUAL?.
neg_works_i32 : -2i32 INT32/->STRING "-2" EQUAL?.
empty_stack_i32 : [INT32/->STRING] TRY UNWRAP 0x04 EQUAL?.

works_i64 : 2i64 INT64/->STRING "2" EQUAL?.
neg_works_i64 : -2i64 INT64/->STRING "-2" EQUAL?.
empty_stack_i64 : [INT64/->STRING] TRY UNWRAP 0x04 EQUAL?.
```
