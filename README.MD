# Blah (**B**lah **L**ooks **A**wful, **H**uh?): Unified Toolchain for Brainfuck

For a long time, Brainfuck has lacked modern tooling — not anymore. blah is:

- A Brainfuck runtime
- A Brainfuck compiler (LLVM backend)
- A Brainfuck package manager

## Quick Start

```sh
cargo install blah
```

Create a new project:

```sh
blah new hello
cd hello
blah run main.bf
```

Run a brainfuck file:

```sh
blah path/to/program.bf
```

## Commands

| Command             | Description           |
| ------------------- | --------------------- |
| `blah new <name>`   | Create a new project. |
| `blah run <file>`   | Run a brainfuck file. |
| `blah <file>`       | Same as above.        |
| `blah build <file>` | Build native binary.  |
| `blah install`      | Install dependencies. |

## bark.toml

```toml
name = "hello"

[features]
# use it via @pkg-name/feature-name in source code
stdout = "stdout.bf"

[dependencies]
stdlib = { git = "https://github.com/example/bf-stdlib" }
utils = { path = "../utils" }
```

Dependencies support two sources:

- git: downloaded via git clone
- path: local path reference

## Package Manager

Blah is a package manager.

In Brainfuck source code, use `@package-name/feature-name` to reference code snippets from installed packages. The behavior is like `include` in C.
