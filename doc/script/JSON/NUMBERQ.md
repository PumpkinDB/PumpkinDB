# JSON/NUMBER?

{% method -%}

Tests binary if it contains a valid JSON number expression

Input stack: `a`

Output stack: `b`

`JSON/NUMBER?` will push `1` if supplied binary contains a valid JSON number
expression, `0` otherwise

{% common -%}

```
PumpkinDB> "1" JSON/NUMBER?
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
works : "1" JSON/NUMBER?.
works_negative : "true" JSON/NUMBER? NOT.
non_json : "z" JSON/NUMBER? NOT.
empty_stack : [JSON/NUMBER?] TRY UNWRAP 0x04 EQUAL?.
```
