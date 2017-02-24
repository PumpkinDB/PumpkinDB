# ?CURSOR/SEEKLAST

{% method -%}

Sets the cursor at the key value pair with the last key that has a given prefix.

Input stack: `cursor prefix`

Output stack: `[key value]` or `[]`

If there are key/value pairs in the database that have their key
prefixed with `prefix`, the `[key value]` pair with the largest key
will be pushed onto the stack. Otherwise, `[]` will be pushed. Useful in conjunction with [UNWRAP](../UNWRAP.md),
[SOME?](../SOMEQ.md) and [NONE?](../NONEQ.md).

{% common -%}

```
PumpkinDB> ["key" HLC CONCAT 1 ASSOC
 "key" HLC CONCAT 2 ASSOC
 "key" HLC CONCAT 3 ASSOC COMMIT] WRITE
[CURSOR "key" ?CURSOR/SEEKLAST] READ UNWRAP SWAP DROP
3
```

{% endmethod %}

## Allocation

Allocates for values to be put onto the stack

## Errors

NoTransaction error if there's no current write transaction

InvalidValue error if the cursor identifier is incorrect or expired

## Tests

```test
lastkey : ["a" HLC CONCAT 0 ASSOC
           "key" HLC CONCAT 1 ASSOC
           "key" HLC CONCAT 2 ASSOC
           "key" HLC CONCAT 3 ASSOC COMMIT] WRITE
          [CURSOR "key" ?CURSOR/SEEKLAST] READ UNWRAP
          3 EQUAL?.
not_lastkey : ["key" HLC CONCAT 1 ASSOC
               "key" HLC CONCAT 2 ASSOC
               "key" HLC CONCAT 3 ASSOC
               "zzzz" HLC CONCAT 4 ASSOC COMMIT] WRITE
          [CURSOR "key" ?CURSOR/SEEKLAST] READ UNWRAP
          3 EQUAL?.
no_key : ["zzzz" HLC CONCAT 4 ASSOC COMMIT] WRITE
          [CURSOR "key" ?CURSOR/SEEKLAST] READ NONE?.
requires_txn : ["1" "1" ?CURSOR/SEEKLAST] TRY UNWRAP 0x08 EQUAL?.
empty_stack : [?CURSOR/SEEKLAST] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : ["a" ?CURSOR/SEEKLAST] TRY UNWRAP 0x04 EQUAL?.
invalid_cursor : [["1" "A" ?CURSOR/SEEKLAST] READ] TRY UNWRAP 0x03 EQUAL?.
```
