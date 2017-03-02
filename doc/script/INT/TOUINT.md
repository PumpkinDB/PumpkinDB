INT->UINT
===

{% method -%}

Converts a `INT` to `UINT`, returning an error if
that is not possible.

Input stack: `a`

Output stack: `b`

`INT->UINT` will push `b`, the `UINT` transformed value of
`a` to the top of the stack.

{% common -%}

```
PumpkinDB> +1 INT->UINT
1
```

{% endmethod %}

## Allocation

Runtime allocation for the `UINT` added to the stack.

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are less than one item on the stack.

[InvalidValue](../errors/InvalidValue.md) error if casting to a `UINT` is impossible.

[InvalidValue](../errors/InvalidValue.md) error if `a` cannot be signed integer

## Tests

```test
works : +1 INT->UINT 1 EQUAL?.
empty_stack : [INT->UINT] TRY UNWRAP 0x04 EQUAL?.
impossible_cast : [-1 INT->UINT] TRY UNWRAP 0x03 EQUAL?.
```
