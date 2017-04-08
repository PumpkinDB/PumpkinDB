# CURSOR/SEEK

{% method -%}

Sets the cursor at the key value pair with a greater or equal key

Input stack: `cursor key`

Output stack: `b`

If there is a key/value pair in the database that has a key
that is greater or equal to `key`, `1` will be pushed onto the stack and the cursor will be moved.
Otherwise, `0` will be pushed and the cursor will not be moved.

{% common -%}

```
PumpkinDB> ["3" "3" ASSOC COMMIT] WRITE [CURSOR DUP "2" CURSOR/SEEK SWAP CURSOR/KEY] READ
1 "3"
```

{% endmethod %}

## Allocation

Allocates for values to be put onto the stack

## Errors

[NoTransaction](../errors/NoValue.md) error if there's no current write transaction

[InvalidValue](../errors/InvalidValue.md) error if the cursor identifier is incorrect or expired

## Tests

```test
works : ["3" "3" ASSOC COMMIT] WRITE [CURSOR DUP "2" CURSOR/SEEK SWAP CURSOR/KEY "3" EQUAL? AND] READ.
seek_end : ["3" "3" ASSOC COMMIT] WRITE [CURSOR DUP "4" CURSOR/SEEK NOT] READ.
requires_txn : ["1" "1" CURSOR/SEEK] TRY UNWRAP 0x08 EQUAL?.
empty_stack : [CURSOR/SEEK] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : [["a" CURSOR/SEEK] TRY] READ UNWRAP 0x04 EQUAL?.
invalid_cursor : [["1" "A" CURSOR/SEEK] READ] TRY UNWRAP 0x03 EQUAL?.
```
