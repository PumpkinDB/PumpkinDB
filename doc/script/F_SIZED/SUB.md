F[size]/SUB
===

{% method -%}

Subtracts two sized floats, either f32 or f64.

Input stack: `a` `b`

Output stack: `c`

`SUB` will push the difference between `b` and `a` on top of the stack.

{% common -%}

```
PumpkinDB> 3.14159f32 2.5f32 F32/SUB
0x40dbc0bf
```

{% endmethod %}

## Allocation

Runtime allocations for decoding numbers and heap allocation
for the result.

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are less than two items on the stack

## Tests

```test
works_32 : 1.0f32 +2.5f32 F32/SUB -1.5f32 EQUAL?.
neg_works_32 : -1.0f32 -2.5f32 F32/SUB 1.5f32 EQUAL?.
neg_pos_works_32 : -1.0f32 2.5f32 F32/SUB -3.5f32 EQUAL?.
int_arg_fails_32 : [1.0f32 1 F32/SUB] TRY UNWRAP 0x03 EQUAL?.
empty_stack_32 : [F32/SUB] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1_32 : [1.0f32 F32/SUB] TRY UNWRAP 0x04 EQUAL?.
works_64 : 1.0f64 +2.5f64 F64/SUB -1.5f64 EQUAL?.
neg_works_64 : -1.0f64 -2.5f64 F64/SUB 1.5f64 EQUAL?.
neg_pos_works_64 : -1.0f64 2.5f64 F64/SUB -3.5f64 EQUAL?.
int_arg_fails_64 : [1.0f64 1 F64/SUB] TRY UNWRAP 0x03 EQUAL?.
empty_stack_64 : [F64/SUB] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1_64 : [1.0f64 F64/SUB] TRY UNWRAP 0x04 EQUAL?.
```
