# rustmd

Parser / compiler for markdown syntax with a focus on memory safety and correctness.

Converts markdown into a syntax tree of tokens with the help of [nom](https://github.com/Geal/nom), a library for writing parsers.

## Memory safety

Achieving memory safety is a core feature of Rust. Rust allows (forces) the programmer to explicitly handle potentially unsafe situations. Eg. an indexation into an array will produce either a value or instance of `Error`. The programmer is heavily encouraged to handle the `Error`-case by. The only other alternative is to call `unwrap` on a potentially unsafe operation. If this operation produces an `Error`, since it's not being  handled the program will immediately panic terminating itself.
Along with borrow checking rules this is one of the primary mechanisms by which Rust avoids overflow and other memory management related vulnerabilities.

For the sake of completeness, this implementation attempts to completely avoid any `unwrap` calls. There are currently like two of them in the code, which I've been too lazy to explicitly handle.

There are no `unsafe`-blocks in the code, which would allow unsafe operations on memory; The implementation is guaranteed to work in a memory safe manner by the Rust compiler.

## Safe parsing



## Difficulties with trees from the perspective of memory safety


## Testing

## The state of the implementation

The current implementation is far enough to illustrate that it's now possible to support the entirety of the markdown syntax by simply defining more `Tokens` and implementing the `Parse`- and `Compile`- traits for them.
So *optimistically* the only thing left to do for a complete markdown parser is manual work, since the logic has been figured out.
