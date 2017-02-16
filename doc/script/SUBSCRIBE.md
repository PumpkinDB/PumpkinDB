# SUBSCRIBE

Subscribes the connection to a topic

Input stack: `topic`
Output stack: ``

`SUBSCRIBE` allows connected client to subscribe any message topic
on the server.

## Allocation

Runtime allocations necessary for the server  

## Errors

[EmptyStack](./ERRORS/EmptyStack.md) error if there is less than one item on the stack

## Examples

```
"topic" SUBSCRIBE =>
```

## Tests

```
```