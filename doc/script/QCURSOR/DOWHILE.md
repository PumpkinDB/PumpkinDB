# ?CURSOR/DOWHILE

{% method -%}

Fetching cursor walker.

Input stack: `cursor closure iterator`

Output stack:

`?CURSOR/DOWHILE` will execute `closure` while it returns `1`,
invoking `iterator` on the `cursor` after each run. The closure
will receive a result of `?CURSOR/CUR` on top of the stack.

{% common -%}

```
PumpkinDB> ["testkey" HLC CONCAT 1 ASSOC
   "testkey" HLC CONCAT 2 ASSOC
   "testkey" HLC CONCAT 3 ASSOC
   "z" HLC CONCAT 4 ASSOC
   COMMIT] WRITE
  [CURSOR 'c SET
   c CURSOR/FIRST? DROP
   c [SWAP DROP 1] 'CURSOR/NEXT? ?CURSOR/DOWHILE] READ
0x01 0x02 0x03 0x04   
```

{% endmethod %}

## Allocation

Allocates for closure composition

## Errors

NoTransaction error if there's no current read or write transaction

InvalidValue error if the cursor identifier is incorrect or expired

## Tests

```test
?cursor_dowhile :
  ["testkey" HLC CONCAT 1 ASSOC
   "testkey" HLC CONCAT 2 ASSOC
   "testkey" HLC CONCAT 3 ASSOC
   "z" HLC CONCAT 4 ASSOC
   COMMIT] WRITE
  [CURSOR 'c SET
   c CURSOR/FIRST? DROP
   c [UNWRAP SWAP DROP 1] 'CURSOR/NEXT? ?CURSOR/DOWHILE] READ
  4 WRAP [1 2 3 4] EQUAL?.
```
