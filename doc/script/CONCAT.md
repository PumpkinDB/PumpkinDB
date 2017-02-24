# CONCAT

{% method -%}

Takes two topmost items and concatenates them, pushes the
result to the top of the stack

Input stack: `a b`

Output stack: `ab`

{% common -%}

```
PumpkinDB> "Hello, " "world!" CONCAT
"Hello, world!"
```

{% endmethod %}

## Allocation

Allocates for a result of concatenation

[EmptyStack](./errors/EmptyStack.md) error if there are less than two items on the stack

## Tests

```test
concat : "Hello, " "world!" CONCAT "Hello, world!" EQUAL?.
concat_requires_two_items_0 : [CONCAT] TRY UNWRAP 0x04 EQUAL?.
concat_requires_two_items_1 : [1 CONCAT] TRY UNWRAP 0x04 EQUAL?.
```
