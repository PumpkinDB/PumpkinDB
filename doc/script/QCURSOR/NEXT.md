# ?CURSOR/NEXT

{% method -%}

Sets the cursor at the next key value

Input stack: `cursor`

Output stack: `[key value]` or `[]`

If there is a next key/value pair in the database, `[key value]` will be pushed onto the stack.
Otherwise, `[]` will be pushed. Useful in conjunction with [UNWRAP](../UNWRAP.md),
[SOME?](../SOMEQ.md) and [NONE?](../NONEQ.md).

{% common -%}

```
PumpkinDB> ["1" "2" ASSOC "2" "2" ASSOC COMMIT] WRITE [CURSOR 'c SET c ?CURSOR/FIRST DROP c CURSOR/NEXT] READ UNWRAP
"2" "2"
```

{% endmethod %}

## Allocation

Allocates for values to be put onto the stack

## Errors

NoTransaction error if there's no current write transaction

InvalidValue error if the cursor identifier is incorrect or expired

## Tests

```test
works : ["1" "2" ASSOC "2" "2" ASSOC COMMIT] WRITE [CURSOR 'c SET c ?CURSOR/FIRST DROP c ?CURSOR/NEXT] READ ["2" "2"] EQUAL?.
requires_txn : ["1" ?CURSOR/NEXT] TRY UNWRAP 0x08 EQUAL?.
empty_stack : [[?CURSOR/NEXT] TRY] READ UNWRAP 0x04 EQUAL?.
invalid_cursor : [["1" ?CURSOR/NEXT] READ] TRY UNWRAP 0x03 EQUAL?.
```
