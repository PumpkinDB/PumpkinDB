# STRING/->UINT

{% method -%}

Convert string to unsigned integer.

Input stack: `numeric-string`

Output stack: `number`

`STRING/->UINT` pushes the number represented by input string onto the stack.

{% common -%}

```
PumpkinDB> "1024" STRING/->UINT.
0x0400
```

{% endmethod %}

## Allocation

Space for string representation of number will be allocated.

## Errors

[EmptyStack](../errors/EmptyStack.md) error if stack is empty.

[InvalidValue](../errors/InvalidValue.md) error if stack value cannot be converted to integer.

## Tests

```test
works : "2" STRING/->UINT 2 EQUAL?.
empty_stack : [STRING/->UINT] TRY UNWRAP 0x04 EQUAL?.
invalid_value : ["NOT A NUM" STRING/->UINT] TRY UNWRAP 0x03 EQUAL?.
```
