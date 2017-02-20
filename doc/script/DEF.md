# DEF

Defines a word with a closure.

Input stack: `c w`

Output stack:

Since it is rather bothersome to keep repeating code over and over,
it'd be nice to be able define words as composites of other for the
scope of the program.

`DEF` allows to define word's program for the scope of the script's
remainder.

`DEF` will put the second topmost item off the stack (`c`) into the
word referenced by top item (`w`)

## Allocation

None.

## Errors

[EmptyStack](./ERRORS/EmptyStack.md) error if there are less than two items on the stack.

It will error if the format of the word is incorrect

It may error if this word is a built-in word that was previously
defined.

## Examples

```
[DUP DUP] 'dup2 DEF 1 dup2 => 1 1 1
```

## Tests

```test
works : [DUP DUP] 'dup2 DEF 1 dup2 3 WRAP 1 1 1 3 WRAP EQUAL?.
empty_stack : [DEF] TRY UNWRAP 0x04 EQUAL?.
empty_stack_1 : ['a DEF] TRY UNWRAP 0x04 EQUAL?.
```