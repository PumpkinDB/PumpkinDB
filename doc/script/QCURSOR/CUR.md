# ?CURSOR/CUR

{% method -%}

Sets the cursor at the current key value

Input stack: `cursor`

Output stack: `[key value]` or `[]`

If there is a current key/value pair set in the cursor, `[key value]` will be pushed onto the stack.
Otherwise, `[]` will be pushed. Useful in conjunction with [UNWRAP](../UNWRAP.md),
[SOME?](../SOMEQ.md) and [NONE?](../NONEQ.md).

{% common -%}

```
PumpkinDB> ["1" "2" ASSOC COMMIT] WRITE [CURSOR 'c SET c ?CURSOR/FIRST DROP c ?CURSOR/CUR] READ UNWRAP
"1" "2"
```

{% endmethod %}

## Allocation

Allocates for values to be put onto the stack

## Errors

[NoTransaction](../errors/NoTransaction.md) error if there's no current write transaction

[InvalidValue](../errors/InvalidValue.md) error if the cursor identifier is incorrect or expired

[EmptyStack](../errors/EmptyStack.md) error if there is less than one item available on the stack

## Tests

```test
works : ["1" "2" ASSOC COMMIT] WRITE [CURSOR 'c SET c ?CURSOR/FIRST DROP c ?CURSOR/CUR] READ ["1" "2"] EQUAL?.
requires_txn : ["1" ?CURSOR/CUR] TRY UNWRAP 0x08 EQUAL?.
empty_stack : [?CURSOR/CUR] TRY UNWRAP 0x04 EQUAL?.
invalid_cursor : [["1" ?CURSOR/CUR] READ] TRY UNWRAP 0x03 EQUAL?.
```
