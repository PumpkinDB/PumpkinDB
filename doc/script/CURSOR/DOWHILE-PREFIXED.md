# CURSOR/DOWHILE-PREFIXED

Fetching cursor walker for a range of keys starting with a prefix.

Input stack: `prefix closure`

Output stack: 

`CURSOR/DOWHILE-PREFIXED` will start the cursor from the first key that is
equal or grater than `prefix` and execute `closure` while it returns `1`,
invoking `CURSOR/NEXT?` on the `cursor` after each run. The closure
will receive a result of `?CURSOR/CUR UNWRAP` on top of the stack, if the
key starts with the `prefix`.

## Allocation

Allocates for closure composition

## Errors

NoTransaction error if there's no current read or write transaction

InvalidValue error if the cursor identifier is incorrect or expired

## Examples

```
PumpkinDB> ["a" HLC CONCAT 0 ASSOC
              "testkey" HLC CONCAT 1 ASSOC
              "testkey" HLC CONCAT 2 ASSOC
              "testkey" HLC CONCAT 3 ASSOC
              "z" HLC CONCAT 4 ASSOC 
              COMMIT] WRITE
             ["testkey" [SWAP DROP 1] CURSOR/DOWHILE-PREFIXED] READ
0x01 0x02 0x03             
```


## Tests

```test
cursor_dowhile_prefixed : 
  ["a" HLC CONCAT 0 ASSOC
   "testkey" HLC CONCAT 1 ASSOC
   "testkey" HLC CONCAT 2 ASSOC
   "testkey" HLC CONCAT 3 ASSOC
   "z" HLC CONCAT 4 ASSOC 
   COMMIT] WRITE
  ["testkey" [SWAP DROP 1] CURSOR/DOWHILE-PREFIXED] READ
  3 WRAP [1 2 3] EQUAL?.
```


