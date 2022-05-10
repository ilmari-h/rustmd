# rustmd

Parser / compiler for markdown syntax with a focus on memory safety and correctness.

Converts markdown into a syntax tree of tokens with the help of [nom](https://github.com/Geal/nom), a library for writing parsers.

## Memory safety

Achieving memory safety is a core feature of Rust. Rust allows (forces) the programmer to explicitly handle potentially unsafe situations. Eg. an indexation into an array will produce either a value or instance of `Error`. The programmer is encouraged to handle the `Error`-case by having to check for an `Error` value before accessing the result of the operation. The only other alternative is to call `unwrap` on a potentially unsafe operation. If this operation produces an `Error`, since it's not being  handled the program will immediately panic terminating itself.
Along with borrow checking rules this is one of the primary mechanisms by which Rust avoids overflow and other memory management related vulnerabilities.

For the sake of completeness, this implementation attempts to completely avoid any `unwrap` calls. There are currently like two of them in the code, which I've been too lazy to explicitly handle.

There are no `unsafe`-blocks in the code, which would allow unsafe operations on memory; The implementation is guaranteed by the Rust compiler to work in a memory safe manner.

## Safe parsing

Writing a parser from scratch would be a project on it's own with a lot of potential for unsafe practices.
The library *nom* was chosen to help with this.
By using *nom*, there's no need to index into a fixed sized string and operate on individual bytes or anything like that, which could potentially cause vulnerabilities.
Instead, *nom* provides a declarative syntax that makes writing safe and correct parsers convinient.

## Testing

Tests were written using Rust's built in assertion macros and testing framework.

Tests can be ran by running: `yarn test`.

## The state of the implementation and self evaluation

The current implementation is far enough to illustrate that it's now possible to support the entirety of the markdown syntax by simply defining more `Tokens` and implementing the `Parse`- and `Compile`- traits for them.
So *optimistically* the only thing left to do for a complete markdown parser is manual work, since the logic has been figured out.

The focus of the project from the POV of secure programming moved from higher level to a lower as the implementation went on.

The main achieved goal of secure programming here is secure practices in accessing memory: there are no unsafe allocations or any potentials for overflow. This proved difficult at times by virtue of the language's compiler being very conservative in what it considers to be safe code. Especially creating and using a syntax tree proved challenging, as tree-structures are a domain where it's easy to have things like dangling pointers and other sources of vulnerabilities.
