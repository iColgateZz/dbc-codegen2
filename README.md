# dbc-codegen2

`dbc-codegen2` is a **DBC code generator for CAN bus messages**.

It parses a `.dbc` file, builds an intermediate representation (IR), and generates strongly typed code for working with CAN frames.

Currently supported targets:

- Rust
- C++

---

## Installation

Install using Cargo from crates.io:

```bash
cargo install dbc-codegen2
```

After installation the CLI is available as:

```bash
dbc-codegen2
```

## Usage
```bash
dbc-codegen2 <COMMAND> <INPUTS> -o <OUTPUT> <FLAGS>
```

### Commands

| Command | Description                              |
| ------- | ---------------------------------------- |
| `parse` | Parse a DBC file and print parsed output |
| `ir`    | Show intermediate representation         |
| `gen`   | Generate code from a DBC file            |

Use `--help` to get more information. For example,

```bash
dbc-codegen2 gen --help
```


### Flags

| Flag    | Description                              |
| ------- | ---------------------------------------- |
| `--no-enum-dedup` | disable enum deduplication |
| `--no-enum-other` | remove `_Other` variant from enums |
| `--zero-zero-range-allows-all` | remove range checks from signals that have `[0\|0]` ranges |

### Code injection

If you use the project as a library and build the `CodegenConfig` manually, you can fill out the `rust_code_injections` map with strings that will be inserted into the generated code. Please take a look at `src/main.rs` main function for an example of code injection.
