# JSON/NULL?

{% method -%}

Tests binary if it contains a valid JSON null expression

Input stack: `a`

Output stack: `b`

`JSON/NULL?` will push `1` if supplied binary contains a valid JSON null
expression, `0` otherwise

{% common -%}

```
PumpkinDB> "null" JSON/NULL?
0x01
PumpkinDB> "1" JSON/NULL?
0x00
```

{% endmethod %}

## Allocation

Allocates for parsing JSON

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are less than one item on the stack

## Tests

```test
works : "null" JSON/NULL?.
works_negative : "1" JSON/NULL? NOT.
non_json : "z" JSON/NULL? NOT.
empty_stack : [JSON/NULL?] TRY UNWRAP 0x04 EQUAL?.
```
