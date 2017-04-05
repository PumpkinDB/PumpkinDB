# TRACE

**Only available in pumpkindb-term**

{% method -%}

Takes the topmost item and sends it back to the terminal as a traced value.

Input stack: `value`
Output stack: none

This instruction is used for debugging purposes. Multiple invocations are possible per script.

{% common -%}

```
PumpkinDB> "start" TRACE 1.
Trace: "start"
0x01
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there is less than one item on the stack

## Tests

No tests, this instruction is only defined in pumpkindb-term