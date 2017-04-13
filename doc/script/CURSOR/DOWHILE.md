# CURSOR/DOWHILE

{% method -%}

Generic cursor walker.

Input stack: `cursor closure iterator`

Output stack:

`CURSOR/DOWHILE` will execute `closure` while it leaves `1` on
top of the stack, invoking `iterator` on the `cursor` after each run. 
The closure should be written with the expectation of the cursor on the top of the stack.

{% common -%}

```
PumpkinDB> ["testkey" HLC CONCAT 1 ASSOC
   "testkey" HLC CONCAT 2 ASSOC
   "testkey" HLC CONCAT 3 ASSOC
   "z" HLC CONCAT 4 ASSOC
   COMMIT] WRITE
  [CURSOR DUP CURSOR/FIRST DROP
     [CURSOR/VAL TRUE] 'CURSOR/NEXT CURSOR/DOWHILE] READ
0x01 0x02 0x03 0x04   
```

{% endmethod %}
## Allocation

Allocates for closure composition

## Errors

[NoTransaction](../errors/NoValue.md) error if there's no current write transaction

[InvalidValue](../errors/InvalidValue.md) error if the cursor identifier is incorrect or expired

## Tests

```test
cursor_dowhile :
  ["testkey" HLC CONCAT 1 ASSOC
   "testkey" HLC CONCAT 2 ASSOC
   "testkey" HLC CONCAT 3 ASSOC
   "z" HLC CONCAT 4 ASSOC
   COMMIT] WRITE
  [CURSOR DUP CURSOR/FIRST DROP
   [CURSOR/VAL TRUE] 'CURSOR/NEXT CURSOR/DOWHILE] READ
  4 WRAP [1 2 3 4] EQUAL?.
```
