# JSON/ARRAY?

{% method -%}

Tests binary if it contains a valid JSON array expression

Input stack: `a`

Output stack: `b`

`JSON/ARRAY?` will push `1` if supplied binary contains a valid JSON array
expression, `0` otherwise

{% common -%}

```
PumpkinDB> "[1,2,3]" JSON/ARRAY?
0x01
PumpkinDB> "1" JSON/ARRAY?
0x00
```

{% endmethod %}

## Allocation

Allocates for parsing JSON

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are less than one item on the stack

## Tests

```test
works : "[1,2,3]" JSON/ARRAY?.
works_negative : "1" JSON/ARRAY? NOT.
non_json : "z" JSON/ARRAY? NOT.
empty_stack : [JSON/ARRAY?] TRY UNWRAP 0x04 EQUAL?.
```
