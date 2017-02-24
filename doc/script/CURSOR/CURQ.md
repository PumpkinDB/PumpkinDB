# CURSOR/CUR?

{% method -%}

Sets the cursor at the current key value

Input stack: `cursor`

Output stack: `b`

If there is a current key/value pair set in the cursor, `1` will be pushed onto the stack.
Otherwise, `0` will be pushed and the cursor will be moved.

{% common -%}

```
PumpkinDB> ["1" "2" ASSOC COMMIT] WRITE [CURSOR 'c SET c ?CURSOR/FIRST DROP c CURSOR/CUR?] READ UNWRAP
1
```

{% endmethod %}

## Allocation

Allocates for values to be put onto the stack

## Errors

NoTransaction error if there's no current write transaction

InvalidValue error if the cursor identifier is incorrect or expired

## Tests

```test
works : ["1" "2" ASSOC COMMIT] WRITE [CURSOR 'c SET c ?CURSOR/FIRST DROP c CURSOR/CUR?] READ.
requires_txn : ["1" CURSOR/CUR?] TRY UNWRAP 0x08 EQUAL?.
empty_stack : [CURSOR/CUR?] TRY UNWRAP 0x04 EQUAL?.
invalid_cursor : [["1" CURSOR/CUR?] READ] TRY UNWRAP 0x03 EQUAL?.
```
