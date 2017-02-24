# DROP

{% method -%}

Drops an item off the top of the stack

Input stack: `a`

Output stack:

{% common -%}

```
PumpkinDB> 0x10 DROP
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if nothing is available on the stack

## Tests

```test
works : 1 DROP DEPTH 0 EQUAL?.
empty_stack : [DROP] TRY UNWRAP 0x04 EQUAL?.
```
