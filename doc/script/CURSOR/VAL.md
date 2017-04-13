# CURSOR/VAL

{% method -%}

Sets the cursor at the current key value

Input stack: `cursor`

Output stack: `val`

If there is a current key/value pair set in the cursor, the value will be pushed onto the stack.

{% common -%}

```
PumpkinDB> ["1" "2" ASSOC COMMIT] WRITE [CURSOR DUP CURSOR/FIRST DROP CURSOR/VAL] READ
"2"
```

{% endmethod %}

## Allocation

Allocates for values to be put onto the stack

## Errors

[NoTransaction](../errors/NoValue.md) error if there's no current write transaction

[InvalidValue](../errors/InvalidValue.md) error if the cursor identifier is incorrect or expired

[NoValue](../errors/NoValue.md) error if the cursor hasn't been positioned.

## Tests

```test
works : ["1" "2" ASSOC COMMIT] WRITE [CURSOR DUP CURSOR/FIRST SWAP CURSOR/VAL "2" EQUAL? AND] READ.
requires_txn : ["1" CURSOR/VAL] TRY UNWRAP 0x08 EQUAL?.
empty_stack : [[CURSOR/VAL] TRY] READ UNWRAP 0x04 EQUAL?.
invalid_cursor : [["1" CURSOR/VAL] READ] TRY UNWRAP 0x03 EQUAL?.
```
