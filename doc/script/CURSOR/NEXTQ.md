# CURSOR/NEXT?

Sets the cursor at the next key value

Input stack: `cursor`

Output stack: `b`

If there is a next key/value pair in the database, `1` will be pushed onto the stack.
Otherwise, `0` will be pushed and the cursor will be moved.

Useful in conjunction with [CURSOR/CUR](../QCURSOR/CUR.md)

## Allocation

Allocates for values to be put onto the stack

## Errors

NoTransaction error if there's no current write transaction

InvalidValue error if the cursor identifier is incorrect or expired

## Examples

```
["1" "2" ASSOC "2" "2" ASSOC COMMIT] WRITE [CURSOR 'c SET c ?CURSOR/FIRST DROP c CURSOR/NEXT?] READ  => 1
```

## Tests

```test
works : ["1" "2" ASSOC "2" "2" ASSOC COMMIT] WRITE [CURSOR 'c SET c ?CURSOR/FIRST DROP c CURSOR/NEXT?] READ.
requires_txn : ["1" CURSOR/NEXT?] TRY UNWRAP 0x08 EQUAL?.
empty_stack : [CURSOR/NEXT?] TRY UNWRAP 0x04 EQUAL?.
invalid_cursor : [["1" CURSOR/NEXT?] READ] TRY UNWRAP 0x03 EQUAL?.
```
