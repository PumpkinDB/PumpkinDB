# CURSOR/POSITIONED?

{% method -%}

Checks if the cursor is positioned

Input stack: `cursor`

Output stack: `b`

If there is a current key/value pair set in the cursor, `1` will be pushed onto the stack.
Otherwise, `0` will be pushed.

{% common -%}

```
PumpkinDB> ["1" "2" ASSOC COMMIT] WRITE [CURSOR CURSOR/POSITIONED?] READ
0
```

{% endmethod %}

## Allocation

Allocates for values to be put onto the stack

## Errors

[NoTransaction](../errors/NoValue.md) error if there's no current write transaction

[InvalidValue](../errors/InvalidValue.md) error if the cursor identifier is incorrect or expired

## Tests

```test
positioned : ["1" "2" ASSOC COMMIT] WRITE [CURSOR DUP CURSOR/FIRST SWAP CURSOR/POSITIONED? AND] READ.
not_positioned :  [CURSOR CURSOR/POSITIONED? NOT] READ.
requires_txn : ["1" CURSOR/POSITIONED?] TRY UNWRAP 0x08 EQUAL?.
empty_stack : [[CURSOR/POSITIONED?] TRY] READ UNWRAP 0x04 EQUAL?.
invalid_cursor : [["1" CURSOR/POSITIONED?] READ] TRY UNWRAP 0x03 EQUAL?.
```
