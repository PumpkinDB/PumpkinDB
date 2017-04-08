# CURSOR/DOWHILE-PREFIXED

{% method -%}

Fetching cursor walker for a range of keys starting with a prefix.

Input stack: `prefix closure`

Output stack:

`CURSOR/DOWHILE-PREFIXED` will start the cursor from the first key that is
equal or grater than `prefix` and execute `closure` while it leaves `1` on the
top of the stack, invoking `CURSOR/NEXT` on the `cursor` after each run. The closure
should be written with an expectation of the cursor on top of the stack.

{% common -%}

```
PumpkinDB> ["a" HLC CONCAT 0 ASSOC
              "testkey" HLC CONCAT 1 ASSOC
              "testkey" HLC CONCAT 2 ASSOC
              "testkey" HLC CONCAT 3 ASSOC
              "z" HLC CONCAT 4 ASSOC
              COMMIT] WRITE
             ["testkey" [CURSOR/VAL TRUE] CURSOR/DOWHILE-PREFIXED] READ
0x01 0x02 0x03             
```

{% endmethod %}

## Allocation

Allocates for closure composition

## Errors

[NoTransaction](../errors/NoValue.md) error if there's no current write transaction

[InvalidValue](../errors/InvalidValue.md) error if the cursor identifier is incorrect or expired

## Tests

```test
cursor_dowhile_prefixed :
  ["a" HLC CONCAT 0 ASSOC
   "testkey" HLC CONCAT 1 ASSOC
   "testkey" HLC CONCAT 2 ASSOC
   "testkey" HLC CONCAT 3 ASSOC
   "z" HLC CONCAT 4 ASSOC
   COMMIT] WRITE
  ["testkey" [CURSOR/VAL TRUE] CURSOR/DOWHILE-PREFIXED] READ
  3 WRAP [1 2 3] EQUAL?.
nextkey_short : ["key" HLC CONCAT 1 ASSOC
               "key" HLC CONCAT 2 ASSOC
               "key" HLC CONCAT 3 ASSOC
               "z" 4 ASSOC COMMIT] WRITE
          ["key" [CURSOR/VAL TRUE] CURSOR/DOWHILE-PREFIXED] READ
          3 WRAP [1 2 3] EQUAL?.
```
