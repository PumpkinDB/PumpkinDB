# $SYSTEM/VERSION

{% method -%}

Pushes PumpkinDB's version on the stack

Input stack: -

Output stack: `vsn`

{% common -%}

```
PumpkinDB> $SYSTEM/VERSION
"0.1"
```

{% endmethod %}

## Allocation

None

## Errors

None

## Tests

```test
works : $SYSTEM/VERSION "0.2.0" EQUAL?.
```
