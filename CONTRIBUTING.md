Our goal is to encourage frictionless contributions to the project. In order to
achieve that, we use Unprotocols [C4 process](https://rfc.unprotocols.org/spec:1/C4).
Please read it, it will answer a lot of questions. Our goal is to merge pull requests
as quickly as possible and make new stable releases regularly. 

In a nutshell, this means:

* We merge pull requests rapidly (try!)
* We are open to diverse ideas
* We prefer code now over consensus later

An additional commit message kind ("tlog", as in "weblog => blog", "gitlog =>
tlog") is encouraged, a commit without files that retains a contextualized
article on the subject. 

The motivation for this is that web is not a reliable place to retain articles
at (hosts go down, content gets deleted, etc.). Nor it's easy to find relevant
pieces with all the noise out there.

What did the contributor think about when he was developing this or that
part? What train of thought was he on?

Keeping the articles in the git log allows to retain them forever (for as long
as there's at least one copy of the repository somewhere) and provide
context to those who really want to learn more about the project. 

It is highly recommended to watch [Pieter Hintjens' talk on building open
source communities](https://www.youtube.com/watch?v=uzxcILudFWM) as well as
read his [book on the same
matter](https://www.gitbook.com/book/hintjens/social-architecture/details).

# Submitting an issue

According to [development process](https://rfc.unprotocols.org/spec:1/C4#24-development-process),
the issue should describe a documented and provable. What this means is that an
issue should trive to have a clear, understandable problem statement. Just like
a patch, it SHOULD be titled "Problem: ..." and have a detailed description
describing evidence behind it, be it a bug or a feature request, or a longer
term "exploratory" issue.

# Preparing a patch

According to [patch requirements](https://rfc.unprotocols.org/spec:1/C4#23-patch-requirements),
the patch should be a minimal and accurate answer to exactly one identified and
agreed problem. A patch commit message must consist of a single short (less
than 50 characters) line stating the problem ("Problem: ...") being solved,
followed by a blank line and then the proposed solution ("Solution: ...").

```
Problem: short problem statement

Optional longer explanation of the problem that this patch
addresses, giving necessary details for the reader to be
able to understand it better.

Solution: explanation of the solution to the problem. Could
be longer than one line.
```

Also, please don't run `rustfmt` (`cargo fmt`) over your patch before committing, unless
you are absolutely sure to include formatting changes that only apply to your actual patch.
Otherwise, it'll make this patch unnecessarily long and might interfere with currently
outstanding PRs or other items in progress. We will run `rustfmt` regularly when nothing
is outstanding to make the code prettier.  This will make everybody's life easier!  
