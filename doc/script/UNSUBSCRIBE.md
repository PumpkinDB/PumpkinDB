# UNSUBSCRIBE

{% method -%}

Unsubscribes the connection from a topic

Input stack: `subscription`

Output stack: ``

`UNSUBSCRIBE` stops connected client to from receiving messages from a
topic it previously subscribed to.

{% common -%}

```
PumpkinDB> "topic" SUBSCRIBE UNSUBSCRIBE
```

{% endmethod %}

## Allocation

Runtime allocations necessary for the server  

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there is less than one item on the stack

## Tests

```test
empty_stack : [UNSUBSCRIBE] TRY UNWRAP 0x04 EQUAL?.
```
