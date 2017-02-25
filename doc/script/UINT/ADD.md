UINT/ADD
===

{% method -%}

Sums two unsigned integers

Input stack: `a` `b`

Output stack: `c`

`AND` will push the sum of `a` and `b` to the top of the stack.

{% common -%}

```
PumpkinDB> 1 2 UINT/ADD
3
```

{% endmethod %}

## Allocation

Runtime allocations for decoding numbers and heap allocation
for the result.

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are less than two items on the stack

## Tests

```test
works : 2 1 UINT/ADD 3 EQUAL?.
empty_stack : [UINT/ADD] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : [1 UINT/ADD] TRY UNWRAP 0x04 EQUAL?.
```
