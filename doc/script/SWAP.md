# SWAP

{% method -%}

Swaps the order of the two topmost items on the stack

Input stack: `a b`

Output stack: `b a`

{% common -%}

```
PumpkinScript> 0x10 0x20 SWAP
0x20 0x10
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than two items on the stack

## Tests

```test
works : 1 2 SWAP.
empty_stack : [SWAP] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : [1 SWAP] TRY UNWRAP 0x04 EQUAL?.
```
