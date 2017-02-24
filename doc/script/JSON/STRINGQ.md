# JSON/STRING?

{% method -%}

Tests binary if it contains a valid JSON string expression

Input stack: `a`

Output stack: `b`

`JSON/STRING?` will push `1` if supplied binary contains a valid JSON string
expression, `0` otherwise

{% common -%}

```
PumpkinDB> "\"a\"" JSON/STRING?
0x01
PumpkinDB> "1" JSON/STRING?
0x00
```

{% endmethod %}

## Allocation

Allocates for parsing JSON

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are less than one item on the stack

## Tests

```test
works : "\"a\"" JSON/STRING?.
works_negative : "1" JSON/STRING? NOT.
non_json : "z" JSON/STRING? NOT.
empty_stack : [JSON/STRING?] TRY UNWRAP 0x04 EQUAL?.
```
