UINT->INT
===

{% method -%}

Converts a `UINT` to `INT`.

Input stack: `a`

Output stack: `b`

`UINT->INT` will push `b`, the `INT` transformed value of
`a` to the top of the stack.

{% common -%}

```
PumpkinDB> 1 UINT->INT
+1
```

{% endmethod %}

## Allocation

Runtime allocation for the `INT` added to the stack.

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are less than one item on the stack.

## Tests

```test
works : 1 UINT->INT +1 EQUAL?.
empty_stack : [UINT->INT] TRY UNWRAP 0x04 EQUAL?.
```
