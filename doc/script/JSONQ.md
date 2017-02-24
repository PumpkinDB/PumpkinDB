# JSON?

{% method -%}

Tests binary if it contains a valid JSON expression

Input stack: `a`

Output stack: `b`

`JSON?` will push `1` if supplied binary contains a valid JSON
expression, `0` otherwise

{% common -%}

```
PumpkinDB> "{}" JSON?
0x01
PumpkinDB> "hello" JSON?
0x00
```

{% endmethod %}

## Allocation

Allocates for parsing JSON

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than one item on the stack

## Tests

```test
works : "{}" JSON?.
works_negative : "hellp" JSON? NOT.
empty_stack : [JSON?] TRY UNWRAP 0x04 EQUAL?.
```
