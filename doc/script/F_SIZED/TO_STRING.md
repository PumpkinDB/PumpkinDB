# F[size]/->STRING

{% method -%}

Convert sized float to string, the size can be f32 or f64.

Input stack: `floating-point-number`

Output stack: `string-of-number`

`F{32,64}->STRING` pushes a string representation of given number to the top of the stack.

{% common -%}

```
PumpkinDB> 3.14159f32 F32/->STRING.
"3.14159"
```

{% endmethod %}

## Allocation

Space for string representation of number will be allocated.

## Errors

[EmptyStack](../errors/EmptyStack.md) error if stack is empty.

[InvalidValue](../errors/InvalidValue.md) error if stack value cannot be converted to integer.

## Tests

```test
works_f32 : 3.14159f32 F32/->STRING "3.14159" EQUAL?.
neg_works_f32 : -2.1f32 F32/->STRING "-2.1" EQUAL?.
empty_stack : [F32/->STRING] TRY UNWRAP 0x04 EQUAL?.
invalid_value : ["NOT A NUM" F32/->STRING] TRY UNWRAP 0x03 EQUAL?.

works_f64 : 3.14159f64 F64/->STRING "3.14159" EQUAL?.
neg_works_f64 : -2.1f64 F64/->STRING "-2.1" EQUAL?.
empty_stack : [F64/->STRING] TRY UNWRAP 0x04 EQUAL?.
invalid_value : ["NOT A NUM" F64/->STRING] TRY UNWRAP 0x03 EQUAL?.
```
