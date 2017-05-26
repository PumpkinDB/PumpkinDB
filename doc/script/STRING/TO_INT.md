# STRING/->INT

{% method -%}

Convert string to signed integer.

Input stack: `numeric-string`

Output stack: `number`

`STRING/->INT` pushes the number represented by input string onto the stack.

{% common -%}

```
PumpkinDB> "1024" STRING/->INT.
0x010400
```

{% endmethod %}

## Allocation

Space for string representation of number will be allocated.

## Errors

[EmptyStack](../errors/EmptyStack.md) error if stack is empty.

[InvalidValue](../errors/InvalidValue.md) error if stack value cannot be converted to integer.

## Tests

```test
works : "2" STRING/->INT +2 EQUAL?.
neg_works : "-2" STRING/->INT -2 EQUAL?.
empty_stack : [STRING/->INT] TRY UNWRAP 0x04 EQUAL?.
invalid_value : ["NOT A NUM" STRING/->INT] TRY UNWRAP 0x03 EQUAL?.
```
