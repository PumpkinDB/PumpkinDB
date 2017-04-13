# CURSOR/SEEKLAST

{% method -%}

Sets the cursor at the key value pair with the last key that has a given prefix.

Input stack: `cursor prefix`

Output stack: `b` 

If there are key/value pairs in the database that have their key
prefixed with `prefix`, then `1` will be pushed onto the stack and the cursor will be
moved to that pair. Otherwise, `0` will be pushed. Due to non-atomicity of the algorithm,
the position of the cursor is undefined in this case and it is highly recommended to reposition it. 

{% common -%}

```
PumpkinDB> ["key" HLC CONCAT 1 ASSOC
 "key" HLC CONCAT 2 ASSOC
 "key" HLC CONCAT 3 ASSOC COMMIT] WRITE
[CURSOR DUP "key" CURSOR/SEEKLAST DROP CURSOR/VAL] READ
3
```

{% endmethod %}

## Allocation

Allocates for values to be put onto the stack

## Errors

[NoTransaction](../errors/NoValue.md) error if there's no current write transaction

[InvalidValue](../errors/InvalidValue.md) error if the cursor identifier is incorrect or expired

## Tests

```test
lastkey : ["a" HLC CONCAT 0 ASSOC
           "key" HLC CONCAT 1 ASSOC
           "key" HLC CONCAT 2 ASSOC
           "key" HLC CONCAT 3 ASSOC COMMIT] WRITE
          [CURSOR DUP "key" CURSOR/SEEKLAST DROP CURSOR/VAL] READ
          3 EQUAL?.
not_lastkey : ["key" HLC CONCAT 1 ASSOC
               "key" HLC CONCAT 2 ASSOC
               "key" HLC CONCAT 3 ASSOC
               "zzzz" HLC CONCAT 4 ASSOC COMMIT] WRITE
          [CURSOR DUP "key" CURSOR/SEEKLAST DROP CURSOR/VAL] READ
          3 EQUAL?.
nextkey_short : ["key" HLC CONCAT 1 ASSOC
               "key" HLC CONCAT 2 ASSOC
               "key" HLC CONCAT 3 ASSOC
               "z" 4 ASSOC COMMIT] WRITE
          [CURSOR DUP "key" CURSOR/SEEKLAST DROP CURSOR/VAL] READ
          3 EQUAL?.
no_key : ["zzzz" HLC CONCAT 4 ASSOC COMMIT] WRITE
          [CURSOR DUP "key" CURSOR/SEEKLAST NOT] READ.
emptydb : [CURSOR DUP "key" CURSOR/SEEKLAST NOT] READ.
requires_txn : ["1" "1" CURSOR/SEEKLAST] TRY UNWRAP 0x08 EQUAL?.
empty_stack : [CURSOR/SEEKLAST] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : ["a" CURSOR/SEEKLAST] TRY UNWRAP 0x04 EQUAL?.
invalid_cursor : [["1" "A" CURSOR/SEEKLAST] READ] TRY UNWRAP 0x03 EQUAL?.
```
