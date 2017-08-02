# \< (PUSH)

{% method -%}

Pushes current stack on the stack of stacks and makes an empty stack
available as the current stack. Used in conjunction with [>](POP.md)

This instruction should not be used lightly as it is primarily an
internal mechanism for managing distinct stacks. It will not be
a part of future Typed PumpkinScript as it will be typed around a single stack.

Input stack: -

Output stack: -

{% common -%}

```
PumpkinDB> 1 2 < 3 >
1 2 
```

{% endmethod %}

## Allocation

Allocates a new empty stack

## Errors

## Tests

```test
works : 1 2 3 4 < DEPTH 0 EQUAL?.
```
