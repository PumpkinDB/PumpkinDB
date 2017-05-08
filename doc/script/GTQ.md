# GT?

{% method -%}

Compares two topmost items lexicographically.

Input stack: `a b`

Output stack: `a`

`GT?` will push `1` if `a` is strictly greater than `b`, `0` otherwise.

{% common -%}

```
PumpkinDB> 0x10 0x20 GT?
0
PumpkinDB> 0x20 0x10 GT?
1
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than two items on the stack

## Tests

```test
less : 0x10 0x20 GT? NOT.
greater : 0x20 0x10 GT?.
equal : 0x10 0x10 GT? NOT.
requires_two_items_0 : [GT?] TRY UNWRAP 0x04 EQUAL?.
requires_two_items_1 : [1 GT?] TRY UNWRAP 0x04 EQUAL?.
```
