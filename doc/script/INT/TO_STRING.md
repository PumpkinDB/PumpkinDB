# INT/->STRING

{% method -%}

Convert a signed integer to string.

Input stack: `number`

Output stack: `string-of-number`

`INT/->STRING` pushes a string representation of given number to the top of the stack.

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

[InvalidValue](../errors/InvalidValue.md) error if stack value cannot be converted to string.

## Tests

```test
works : +2 INT/->STRING "2" EQUAL?.
neg_works : -2 INT/->STRING "-2" EQUAL?.
empty_stack : [INT/->STRING] TRY UNWRAP 0x04 EQUAL?.
invalid_value : ["NOT A NUM" INT/->STRING] TRY UNWRAP 0x03 EQUAL?.
```
