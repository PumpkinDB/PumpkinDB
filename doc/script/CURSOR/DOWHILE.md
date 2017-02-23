# CURSOR/DOWHILE

Generic cursor walker.

Input stack: `cursor closure iterator`

Output stack: 

`CURSOR/DOWHILE` will execute `closure` while it returns `1`,
invoking `iterator` on the `cursor` after each run. The closure
is not expected to receive anything on the stack.

## Allocation

Allocates for closure composition

## Errors

NoTransaction error if there's no current read or write transaction

InvalidValue error if the cursor identifier is incorrect or expired

## Examples

```
PumpkinDB> ["testkey" HLC CONCAT 1 ASSOC
   "testkey" HLC CONCAT 2 ASSOC
   "testkey" HLC CONCAT 3 ASSOC
   "z" HLC CONCAT 4 ASSOC 
   COMMIT] WRITE
  [CURSOR 'c SET
   c CURSOR/FIRST? DROP
   c [?CURSOR/CUR UNWRAP SWAP DROP 1] 'CURSOR/NEXT? CURSOR/DOWHILE] READ
0x01 0x02 0x03 0x04   
```


## Tests

```test
cursor_dowhile : 
  ["testkey" HLC CONCAT 1 ASSOC
   "testkey" HLC CONCAT 2 ASSOC
   "testkey" HLC CONCAT 3 ASSOC
   "z" HLC CONCAT 4 ASSOC 
   COMMIT] WRITE
  [CURSOR 'c SET
   c CURSOR/FIRST? DROP
   c [?CURSOR/CUR UNWRAP SWAP DROP 1] 'CURSOR/NEXT? CURSOR/DOWHILE] READ
  4 WRAP [1 2 3 4] EQUAL?.
```
