# CURSOR

{% method -%}

Creates a read cursor

Input stack: -

Output stack: `cursor`

This is the primary way of navigating the database. This instruction
creates a cursor in a given transactional context and pushes its
identifier onto the stack.

Only valid within [WRITE's](WRITE.md) or [READ](READ.md) scope.

{% common -%}

```
PumpkinDB> [CURSOR 'c SET ...] READ
```
{% endmethod %}

## Allocation

None

## Errors

NoTransaction error if there's no current write transaction

## Tests

```test
requires_txn : [CURSOR] TRY UNWRAP 0x08 EQUAL?.
```
