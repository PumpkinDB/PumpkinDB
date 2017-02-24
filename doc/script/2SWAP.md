# 2SWAP

{% method -%}

Swaps the order of the top two pairs.

Input stack: `a b c d`

Output stack: `c d a b`

{% common -%}

```
PumpkinDB> 0x00 0x00 0x10 0x20 2SWAP
0x20 0x10 0x00 0x00
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than four items on the stack

## Tests

```test
works : 1 2 3 4 2SWAP STACK [3 4 1 2] EQUAL?.
empty_stack : [2SWAP] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : [1 2SWAP] TRY UNWRAP 0x04 EQUAL?.
empty_stack_2 : [2 1 2SWAP] TRY UNWRAP 0x04 EQUAL?.
empty_stack_3 : [3 1 2SWAP] TRY UNWRAP 0x04 EQUAL?.
```
