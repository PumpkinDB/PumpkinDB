# CURSOR/NEXT

{% method -%}

Sets the cursor at the next key value

Input stack: `cursor`

Output stack: `b`

If there is a next key/value pair in the database, `1` will be pushed onto the stack and the cursor will be moved.
Otherwise, `0` will be pushed and the cursor will not be moved.

Useful in conjunction with [CURSOR/CUR](../QCURSOR/CUR.md)

{% common -%}

```
PumpkinDB> ["1" "2" ASSOC "2" "2" ASSOC COMMIT] WRITE [CURSOR DUP CURSOR/FIRST DROP DUP CURSOR/NEXT DROP CURSOR/KEY] READ
"2"
```

{% endmethod %}

## Allocation

Allocates for values to be put onto the stack

## Errors

[NoTransaction](../errors/NoValue.md) error if there's no current write transaction

[InvalidValue](../errors/InvalidValue.md) error if the cursor identifier is incorrect or expired

## Tests

```test
works : ["1" "2" ASSOC "2" "2" ASSOC COMMIT] WRITE [CURSOR DUP CURSOR/FIRST SWAP CURSOR/NEXT AND] READ.
end : ["1" "2" ASSOC "2" "2" ASSOC COMMIT] WRITE [CURSOR DUP CURSOR/LAST SWAP CURSOR/NEXT NOT AND] READ.
requires_txn : ["1" CURSOR/NEXT] TRY UNWRAP 0x08 EQUAL?.
empty_stack : [[CURSOR/NEXT] TRY] READ UNWRAP 0x04 EQUAL?.
invalid_cursor : [["1" CURSOR/NEXT] READ] TRY UNWRAP 0x03 EQUAL?.
```
