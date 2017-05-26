# UINT/->STRING

{% method -%}

Convert an unsigned integer to string.

Input stack: `number`

Output stack: `string-of-number`

`INT/->STRING` pushes a string representation of given number to the top of the stack.

{% common -%}

```
PumpkinDB> 1024 UINT/->STRING.
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
works : 2 UINT/->STRING "2" EQUAL?.
empty_stack : [UINT/->STRING] TRY UNWRAP 0x04 EQUAL?.
```
