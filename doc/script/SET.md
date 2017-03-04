# SET

{% method -%}

Sets a word to value.

Input stack: `v w`

Output stack:

Since it is rather bothersome to keep certain values (like handles
or strings) around by manipulating the stack, it'd be nice to be able
to refer to them directly.

`SET` allows to define a value of a word for the scope of the script's
remainder.

`SET` will put the second topmost item off the stack (`v`) into the
word referenced by top item (`w`)

{% common -%}

```
PumpkinDB> "key" 'key SET [key "value" ASSOC COMMIT] WRITE [key RETR] READ
"value"
```

{% endmethod %}

## Allocation

Allocates on the heap for the binary form definition.

## Errors

[EmptyStack](./errors/EmptyStack.md) error if there are less than two items on the stack

It will error if the format of the word is incorrect

It may error if this word is a built-in word that was previously
defined.

## Tests

```test
works : 1 'val SET 1 val.
empty_stack : [SET] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : ['a SET] TRY UNWRAP 0x04 EQUAL?.
```
