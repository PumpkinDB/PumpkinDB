# PUBLISH

{% method -%}

Publishes data to a topic

Input stack: `data topic`

Output stack:

This instruction is the way to communicate with the rest of the
actors in the database. It will send a data message to a particular
topic, pushing nothing back to the stack. All topic subscribers will
receive it.

{% common -%}

```
PumpkinDB> "Hi" "MAIN" PUBLISH
```

{% endmethod %}

## Allocation

Allocates for sending data copies.

## Errors

[EmptyStack](./errors/EmptyStack.md) error if stack is less than two items on the stack.

## Tests

```test
empty_stack : [PUBLISH] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : [1 PUBLISH] TRY UNWRAP 0x04 EQUAL?.
```
