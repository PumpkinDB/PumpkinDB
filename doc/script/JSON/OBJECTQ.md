# JSON/OBJECT?

{% method -%}

Tests binary if it contains a valid JSON object expression

Input stack: `a`

Output stack: `b`

`JSON/OBJECT?` will push `1` if supplied binary contains a valid JSON object
expression, `0` otherwise

{% common -%}

```
PumpkinDB> "{}" JSON/OBJECT?
0x01
PumpkinDB> "1" JSON/OBJECT?
0x00
```

{% endmethod %}

## Allocation

Allocates for parsing JSON

## Errors

[EmptyStack](../errors/EmptyStack.md) error if there are less than one item on the stack

## Tests

```test
works : "{}" JSON/OBJECT?.
works_negative : "1" JSON/OBJECT? NOT.
empty_stack : [JSON/OBJECT?] TRY UNWRAP 0x04 EQUAL?.
```
