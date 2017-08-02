# WRITE

{% method -%}

Evaluates code in a context of a new write transaction

Input stack: `code`

Output stack: result of `code` evaluation

This instruction is the only way one can write to the database, meaning
instructions like [ASSOC](ASSOC.md) are only possible in the context of
a WRITE. If changes are to be saved, [COMMIT](COMMIT.md) has to be
used as well. Read-transaction related instructions (such as [RETR](RETR.md))
can also be used.

The total number of simultaneous write transactions is limited to one.

`WRITE` evaluates the closure on the current stack.

{% common -%}

```
PumpkinDB> ["hi" "there" ASSOC COMMIT] WRITE
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
evals : [1] WRITE.
invalid_code : [1 WRITE] TRY UNWRAP 0x05 EQUAL?.
empty_stack : [WRITE] TRY UNWRAP 0x04 EQUAL?.
nested_writes_shouldnt_work_for_now : [[[] WRITE] TRY] WRITE UNWRAP 0x09 EQUAL?.
read_nested_writes_shouldnt_work_for_now : [[[] WRITE] TRY] READ UNWRAP 0x09 EQUAL?.
```
