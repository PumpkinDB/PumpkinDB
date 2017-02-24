# UNSUBSCRIBE

{% method -%}

Unsubscribes the connection from a topic

Input stack: `topic`

Output stack: ``

`UNSUBSCRIBE` stops connected client to from receiving messages from a
topic it previously subscribed to.

{% common -%}

```
PumpkinDB> "topic" SUBSCRIBE
```

{% endmethod %}

## Allocation

Runtime allocations necessary for the server  

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there is less than one item on the stack

## Tests

No tests defined as this functionality is currently provided by the server,
not the scheduler.
