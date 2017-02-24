# JSON/BOOLEAN?

{% method -%}

Tests binary if it contains a valid JSON boolean expression

Input stack: `a`

Output stack: `b`

`JSON/BOOLEAN?` will push `1` if supplied binary contains a valid JSON boolean
expression, `0` otherwise

{% common -%}

```
PumpkinDB> "true" JSON/BOOLEAN?
0x01
PumpkinDB> "1" JSON/BOOLEAN?
0x00
```

{% endmethod %}

## Allocation

Allocates for parsing JSON

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are less than one item on the stack

## Tests

```test
works : "true" JSON/BOOLEAN?.
works_negative : "1" JSON/BOOLEAN? NOT.
non_json : "z" JSON/BOOLEAN? NOT.
empty_stack : [JSON/BOOLEAN?] TRY UNWRAP 0x04 EQUAL?.
```
