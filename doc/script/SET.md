# SET

Sets a word value.

Input stack: `c`

Output stack:

Since it is rather bothersome to keep certain values (like handles
or strings) around by manipulating the stack, it'd be nice to be able
to refer to them directly.

`SET` allows to define a value of a word for the scope of the script's
remainder. 

It's syntax is rather interesting. It has two forms, first one is this: 

```
[<word name> : ...code...] SET
```

In this form, `SET` does not evaluate the expression
after the colon, it simply stores it. In effect, each time <word name>
is called, it's re-evaluated again.

The second form is this:

```
[<word name> = ...code...] SET
```

This form immediately evaluates the expression after the equal sign and
stores it. In effect, each time <word name> is called, the same value
will be returned.



## Allocation

The second (immediate evaluation) form allocates runtime memory
 for injecting binding code. The other form does not allocate.

## Errors

EmptyStack error if there are less than two items on the stack

It will error if the format of the closure is incorrect

It may error if this word is a built-in word that was previously
defined.

## Examples

```
[key : "MyKey"] SET [key "value" ASSOC COMMIT] WRITE [key RETR] READ => "value"
```

## Tests

```
[key : "MyKey"] key => "MyKey"
```