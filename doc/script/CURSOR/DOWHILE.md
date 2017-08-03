# CURSOR/DOWHILE

{% method -%}

Generic cursor walker.

Input stack: `cursor closure iterator`

Output stack:

`CURSOR/DOWHILE` will execute `closure` while it leaves `1` on
top of the stack, invoking `iterator` on the `cursor` after each run. 
The closure should be written with the expectation of the cursor on the top of the stack.

`CURSOR/DOWHILE` evaluates the closure on a new stack, popping the
current stack back after each evaluation.

{% common -%}

```
PumpkinDB> ["testkey" HLC CONCAT 1 ASSOC
   "testkey" HLC CONCAT 2 ASSOC
   "testkey" HLC CONCAT 3 ASSOC
   "z" HLC CONCAT 4 ASSOC
   COMMIT] WRITE
  [CURSOR DUP CURSOR/FIRST DROP
     [CURSOR/VAL >Q TRUE] 'CURSOR/NEXT CURSOR/DOWHILE] READ Q> Q> Q>
0x04 0x03 0x02 0x01
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
   [CURSOR/VAL >Q TRUE] 'CURSOR/NEXT CURSOR/DOWHILE] READ
  Q> Q> Q> Q> 
  4 WRAP [4 3 2 1] EQUAL?.
```
