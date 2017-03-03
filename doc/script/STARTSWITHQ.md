# STARTSWITH?

{% method -%}

Tests if binary starts with another binary

Input stack: `a b`

Output stack: `c`

`STARTSWITH?` pushes `1` if binary `a` starts with binary `b`,
`0` otherwise.

{% common -%}

```
PumpkinDB> "ab" "a" STARTSWITH?
0x01
```

{% endmethod %}

## Allocation

Runtime allocation

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than two items on the stack

## Tests

```test
works : "ab" "a" STARTSWITH?.
works_negative : "ab" "c" STARTSWITH? NOT.
smaller_val : "a" "ab" STARTSWITH? NOT.
empty_stack : [STARTSWITH?] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : ["a" STARTSWITH?] TRY UNWRAP 0x04 EQUAL?.
```
