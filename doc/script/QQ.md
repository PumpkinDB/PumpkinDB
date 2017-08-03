# Q?

Pronounced "queued"

{% method -%}

Checks if there any elements in the queue and pushes
a corresponding boolean onto the top of the stack. Does
not alter the queue.

This instruction is useful for draining the queue (as
shown in the example) 

Input stack: -

Output stack: boolean

{% common -%}

```
PumpkinDB> 1 >Q 2 >Q 3 >Q [Q? [Q< TRACE TRUE] [FALSE] IFELSE] DOWHILE
Trace: 0x01
Trace: 0x02
Trace: 0x03
```

{% endmethod %}

## Allocation

None

## Errors

## Tests

```test
empty : Q? FALSE EQUAL?.
non_empty : 1 >Q Q?.
```
