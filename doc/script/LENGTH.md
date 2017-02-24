# LENGTH

{% method -%}

Puts the length of the top item on the stack back to the top of the stack

Input stack: `a`

Output stack: `b`

`LENGTH` pops a top item off the stack and pushes its length back to the
top of the stack.

{% common -%}

```
PumpkinDB> "Hello" LENGTH
5
```

{% endmethod %}

## Allocation

Allocates for the result of the item size calculation

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are no items on the stack

## Tests

```test
works : "123" LENGTH 3 EQUAL?.
empty_stack : [LENGTH] TRY UNWRAP 0x04 EQUAL?.
```
