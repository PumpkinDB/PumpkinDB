INT?
===

{% method -%}

Checks if the topmost item is `INT`

Input stack: `a`

Output stack: `b`

Pushes `1` to the top of the stack if the topmost item
on the stack is `INT`, otherwise `0`.

{% common -%}

```
PumpkinDB> +1 INT?
1
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are less than one item on the stack.

## Tests

```test
works : +1 INT? 1 EQUAL?.
false : 1 INT? 0 EQUAL?.
```
