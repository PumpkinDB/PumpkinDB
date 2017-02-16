# SEND

Sends data to a topic

Input stack: `data topic`

Output stack:

This word is the way to communicate with the rest of the
actors in the database. It will send a data message to a particular
topic, pushing nothing back to the stack. All topic subscribers will
receive it.

## Allocation

Allocates for sending data copies.

## Errors

[EmptyStack](./ERRORS/EmptyStack.md) error if stack is less than two items on the stack.


## Examples

```
"Hi" "MAIN" SEND => 
```
  
## Tests

```
"Hi" "MAIN" SEND => 
```
