# SET

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

If the `scoped_dictionary` feature has been enabled, the definition
is valid for the scope of the closure, or the rest of the program if
used outside of a closure. Otherwise, the definition is valid for the
rest of the program, unless overridden.

## Allocation

Allocates on the heap for the binary form definition.

## Errors

[EmptyStack](./ERRORS/EmptyStack.md) error if there are less than two items on the stack

It will error if the format of the word is incorrect

It may error if this word is a built-in word that was previously
defined.

## Examples

```
[ASSOC COMMIT] 'assoc_and_commit SET ["key" "value" assoc_and_commit] WRITE ["key" RETR] READ => "value"
[CURSOR 'c SET c CURSOR/FIRST c CURSOR/NEXT] READ => <key> <val> <key> <val>
```

## Tests

```
[DUP DUP] 'key SET "MyKey" key => "MyKey" "MyKey" "MyKey"
DEPTH 'depth SET 1 2 3 depth => 0
```