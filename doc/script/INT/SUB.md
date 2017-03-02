INT/SUB
===

{% method -%}

Subtracts one signed integer from another

Input stack: `a` `b`

Output stack: `c`

`SUB` will subtract of `b` from `a` and push it to the top of the stack.

{% common -%}

```
PumpkinDB> +2 +1 INT/SUB
+1
```

{% endmethod %}

## Allocation

Runtime allocations for decoding numbers and heap allocation
for the result.

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are less than two items on the stack

[InvalidValue](../errors/InvalidValue.md) error if `a` or `b` cannot be signed integers

## Tests

```test
works : +2 +1 INT/SUB +1 EQUAL?.
negative_value : +1 +2 INT/SUB -1 EQUAL?.
empty_stack : [INT/SUB] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : [+1 INT/SUB] TRY UNWRAP 0x04 EQUAL?.
```
