# READ

Evaluates code in a context of a new write transaction

Input stack: `code`

Output stack: result of `code` evaluation

This word is the only way one can write to the database, meaning
words like [RETR](RETR.md) are only possible in the context of
a READ. 

## Allocation

Will allocate for `code` appended with an internal transaction end
marker word.

## Errors

[EmptyStack](./ERRORS/EmptyStack.md) error if stack is less than two items on the stack.

[DatabaseError](./ERRORS/DatabaseError.md) error if there's a problem with underlying storage.

[Decoding error](./ERRORS/DECODING.md) error if the code is undecodable.


## Examples

```
["hi" RETR] READ 
```
  
## Tests

```test
evals : [1] READ.
invalid_code : [1 READ] TRY UNWRAP 0x05 EQUAL?.
empty_stack : [READ] TRY UNWRAP 0x04 EQUAL?. 
```
