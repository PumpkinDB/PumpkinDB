# SUBSCRIBE

{% method -%}

Subscribes the connection to a topic

Input stack: `topic`

Output stack: `subscription`

`SUBSCRIBE` allows connected client to subscribe any message topic
on the server. Pushes subscription identifier back to the top of the stack.

{% common -%}

```
PumpkinDB> "topic" SUBSCRIBE
0xea60d7d866144e0184ac8c9b462d0737
```

{% endmethod %}

## Allocation

Runtime allocations necessary for the server  

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there is less than one item on the stack

## Tests

```test
empty_stack : [SUBSCRIBE] TRY UNWRAP 0x04 EQUAL?.
```
