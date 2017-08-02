# \> (POP)

{% method -%}

Pops the topmost stack off the stack of stacks and makes it a current
stack. Previously current stack is discarded. Used in conjunction with [<](PUSH.md)

Input stack: -

Output stack: -

This instruction should not be used lightly as it is primarily an
internal mechanism for managing distinct stacks. It will not be
a part of future Typed PumpkinScript as it will be typed around a single stack.

{% common -%}

```
PumpkinDB> 1 2 < 3 >
1 2 
```

{% endmethod %}

## Allocation

None

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are no stacks that were previously
pushed onto the stack of stacks.

## Tests

```test
restore : 1 2 3 4 < < > > DEPTH 4 EQUAL?.
empty_stack : [>] TRY UNWRAP 0x04 EQUAL?.
```
