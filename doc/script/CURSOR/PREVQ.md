# CURSOR/PREV?

{% method -%}

Sets the cursor at the previous key value

Input stack: `cursor`

Output stack: `b`

If there is a previous key/value pair in the database, `1` will be pushed onto the stack.
Otherwise, `0` will be pushed and the cursor will be moved.

Useful in conjunction with [CURSOR/CUR](../QCURSOR/CUR.md)

{% common -%}

```
PumpkinDB> ["1" "2" ASSOC "2" "2" ASSOC COMMIT] WRITE [CURSOR 'c SET c ?CURSOR/LAST DROP c CURSOR/PREV?] READ
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
works : ["1" "2" ASSOC "2" "2" ASSOC COMMIT] WRITE [CURSOR 'c SET c ?CURSOR/LAST DROP c CURSOR/PREV?] READ.
requires_txn : ["1" CURSOR/PREV?] TRY UNWRAP 0x08 EQUAL?.
empty_stack : [[CURSOR/PREV?] TRY] READ UNWRAP 0x04 EQUAL?. 
invalid_cursor : [["1" CURSOR/PREV?] READ] TRY UNWRAP 0x03 EQUAL?.
```
