# READ

{% method -%}

Evaluates code in a context of a new read transaction

Input stack: `code`

Output stack: result of `code` evaluation

This instruction is one of two ways one can read from the database.
Instructions like [RETR](RETR.md) are only possible in the context of
a READ or a [WRITE](WRITE.md).

The total number of simultaneous read transactions is limited to 126.

`READ` evaluates the closure on the current stack.

{% common -%}

```
PumpkinDB> ["hi" RETR] READ
```

{% endmethod %}

## Allocation

Will allocate for `code` appended with an internal transaction end
marker instruction.

## Errors

[EmptyStack](./errors/EmptyStack.md) error if stack is less than one item on the stack.

[DatabaseError](./errors/DatabaseError.md) error if there's a problem with underlying storage.

[Decoding error](./errors/DECODING.md) error if the code is undecodable.

## Tests

```test
evals : [1] READ.
invalid_code : [1 READ] TRY UNWRAP 0x05 EQUAL?.
empty_stack : [READ] TRY UNWRAP 0x04 EQUAL?.
```
