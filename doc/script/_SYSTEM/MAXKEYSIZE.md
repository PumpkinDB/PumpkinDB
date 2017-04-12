# $SYSTEM/MAXKEYSIZE

{% method -%}

Pushes PumpkinDB's maximum key size on the stack as an unsigned integer (UINT). 

Currently, this limit comes from underlying LMDB storage and can only be changed
with some tweaking compile-time only.

Input stack: -

Output stack: `uint`

{% common -%}

```
PumpkinDB> $SYSTEM/MAXKEYSIZE
0x01ff
```

{% endmethod %}

## Allocation

Currently allocates on heap to put the value on the stack.

## Errors

None

## Tests

```test
works : $SYSTEM/MAXKEYSIZE 511 EQUAL?.
```
