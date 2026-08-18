# dbc-codegen2

`dbc-codegen2` is a DBC code generator for CAN bus messages.

It reads one or more `.dbc` files, parses them with [`can-dbc`](https://crates.io/crates/can-dbc), builds an intermediate representation, validates and normalizes that representation, and then generates strongly typed source code for working with CAN frames.

The generator currently supports:

- Rust output (`.rs`)
- C++ output (`.hpp`)
- no-std compatible output
- plain CAN messages
- simple multiplexed messages with one multiplexor signal per message
- enum generation from DBC value descriptions
- generating tests for generated Rust code
- separate per-message C++ headers
- Rust code injection at selected locations

Unsupported DBC features currently include:

- extended multiplexing symbols
- messages with more than one multiplexor signal
- signals that are both multiplexors and multiplexed signals

## Installation

Install the command-line tool with Cargo:

```bash
cargo install dbc-codegen2
```

After installation, the executable is available as:

```bash
dbc-codegen2
```

You can also use the crate as a Rust library by adding it to your `Cargo.toml`:

```toml
[dependencies]
dbc-codegen2 = "*"
```

## Command-line usage

General form:

```bash
dbc-codegen2 <COMMAND> [OPTIONS]
```

Get top-level help:

```bash
dbc-codegen2 --help
```

Get help for a specific command:

```bash
dbc-codegen2 gen --help
```

## Commands

### `parse`

Parse a DBC file and write the parsed `can-dbc` message data as debug output.

```bash
dbc-codegen2 parse input.dbc -o parsed.txt
```

### `ir`

Parse a DBC file, build the internal IR, and write the IR as debug output.

```bash
dbc-codegen2 ir input.dbc -o ir.txt
```

### `gen`

Generate Rust or C++ code from one or more DBC files.

```bash
dbc-codegen2 gen input.dbc -o generated --lang rust
```

When multiple input files are passed, the parser output is merged before IR construction and validation:

```bash
dbc-codegen2 gen a.dbc b.dbc c.dbc -o vehicle_can.rs
```

Arguments and options:

| Argument / option | Description | Default |
| --- | --- | --- |
| `inputs...` | One or more input DBC file paths | required |
| `-o, --output` | Output path. The extension is replaced with the target language extension. | `data/generated.rs` |
| `-l, --lang` | Target language. Supported values: `rust`, `cpp`. | `rust` |
| `--no-enum-other` | Do not generate the fallback `_Other(...)` variant for signal value enums. Unknown enum values then become errors. | false |
| `--no-enum-dedup` | Disable signal value enum deduplication. By default, equal enums are shared. | false |
| `--allow-unrestricted-ranges` | Treat DBC physical ranges written as `[0\|0]` as unconstrained. | false |
| `--test` | Generate message tests. Rust output emits `#[cfg(test)]` tests; C++ output emits a `generated_tests::run_all()` harness. | false |
| `--separate` | For C++ output, generate a shared common header, one header per message, and an aggregate header. Rust output is unchanged. | false |

Examples:

Generate Rust:

```bash
dbc-codegen2 gen vehicle.dbc -o src/generated_can
```

Generate Rust with generated tests:

```bash
dbc-codegen2 gen vehicle.dbc -o src/generated_can --test
```

Generate C++:

```bash
dbc-codegen2 gen vehicle.dbc -o include/generated_can --lang cpp
```

Generate C++ with generated tests:

```bash
dbc-codegen2 gen vehicle.dbc -o include/generated_can --lang cpp --test
```

Generate C++ with one header per message:

```bash
dbc-codegen2 gen vehicle.dbc -o include/generated_can --lang cpp --separate
```

Generate Rust while rejecting unknown enum values:

```bash
dbc-codegen2 gen vehicle.dbc -o src/generated_can --no-enum-other
```

Generate from multiple DBC files:

```bash
dbc-codegen2 gen network_a.dbc network_b.dbc -o generated/network
```

## Library usage

Library entry point is `CodegenPipeline::run`, using `CodegenConfig`.

```rust
use dbc_codegen2::{CodegenPipeline, CodegenConfig, Language};
use std::collections::HashMap;

fn main() -> anyhow::Result<()> {
    let config = CodegenConfig {
        inputs: vec!["vehicle.dbc".to_string()],
        output: "src/generated_can".to_string(),
        lang: Language::Rust,
        enum_other: true,
        enum_dedup: true,
        allow_unrestricted_ranges: false,
        rust_code_injections: HashMap::new(),
        cpp_code_injections: HashMap::new(),
        generate_tests: true,
        separate: false,
    };

    CodegenPipeline::run(config)
}
```

### Rust code injection

Rust code injection lets library users insert extra Rust tokens before selected generated items. This is useful for adding derives to generated types.

Available injection points are:

| Injection point | Inserted before |
| --- | --- |
| `RustCodeInjectionPoint::MessageStruct` | every generated message struct |
| `RustCodeInjectionPoint::MessageEnum` | the generated top-level message enum |
| `RustCodeInjectionPoint::SignalValueEnum` | every generated signal value enum |
| `RustCodeInjectionPoint::MuxEnum` | every generated mux enum |
| `RustCodeInjectionPoint::MuxVariantStruct` | every generated mux variant struct |
| `RustCodeInjectionPoint::ErrorEnum` | the generated `CanError` enum |
| `RustCodeInjectionPoint::Getter` | signal getters |
| `RustCodeInjectionPoint::Setter` | signal setters |

Example:

```rust
use dbc_codegen2::{CodegenPipeline, CodegenConfig, Language, RustCodeInjectionPoint};
use std::collections::HashMap;

fn main() -> anyhow::Result<()> {
    let mut config = CodegenConfig {
        inputs: vec!["vehicle.dbc".to_string()],
        output: "src/generated_can".to_string(),
        lang: Language::Rust,
        enum_other: true,
        enum_dedup: true,
        allow_unrestricted_ranges: false,
        rust_code_injections: HashMap::new(),
        cpp_code_injections: HashMap::new(),
        generate_tests: false,
        separate: false,
    };

    config.add_rust_code_injection(
        RustCodeInjectionPoint::SignalValueEnum,
        "#[derive(serde::Serialize, serde::Deserialize)]",
    );

    config.add_rust_code_injection(
        RustCodeInjectionPoint::MessageStruct,
        "#[cfg_attr(feature = \"defmt\", derive(defmt::Format))]",
    );

    CodegenPipeline::run(config)
}
```

### C++ code injection

C++ code injection lets library users insert extra C++ before selected generated items or inside generated classes.

Available injection points are:

| Injection point | Inserted at |
| --- | --- |
| `CppCodeInjectionPoint::Header` | after generated `#include`s, before generated declarations |
| `CppCodeInjectionPoint::Footer` | after all generated declarations |
| `CppCodeInjectionPoint::ErrorEnum` | before the generated `CanError` enum |
| `CppCodeInjectionPoint::MessageVariant` | before the generated top-level `CanMsg` variant alias |
| `CppCodeInjectionPoint::SignalValueEnum` | before every generated signal value enum |
| `CppCodeInjectionPoint::MuxVariant` | before every generated mux variant alias |
| `CppCodeInjectionPoint::MuxVariantClass` | before every generated mux variant class |
| `CppCodeInjectionPoint::MuxVariantClassPublic` | inside every generated mux variant class, after `public:` |
| `CppCodeInjectionPoint::MuxVariantClassPrivate` | inside every generated mux variant class, after `private:` |
| `CppCodeInjectionPoint::MessageClass` | before every generated message class |
| `CppCodeInjectionPoint::MessageClassPublic` | inside every generated message class, after `public:` |
| `CppCodeInjectionPoint::MessageClassPrivate` | inside every generated message class, after `private:` |

Example:

```rust
use dbc_codegen2::{CodegenPipeline, CodegenConfig, CppCodeInjectionPoint, Language};
use std::collections::HashMap;

fn main() -> anyhow::Result<()> {
    let mut config = CodegenConfig {
        inputs: vec!["vehicle.dbc".to_string()],
        output: "include/generated_can.hpp".to_string(),
        lang: Language::Cpp,
        enum_other: true,
        enum_dedup: true,
        allow_unrestricted_ranges: false,
        rust_code_injections: HashMap::new(),
        cpp_code_injections: HashMap::new(),
        generate_tests: false,
        separate: false,
    };

    config.add_cpp_code_injection(
        CppCodeInjectionPoint::Header,
        "#include \"generated_can_extra.hpp\"",
    );

    config.add_cpp_code_injection(
        CppCodeInjectionPoint::MessageClassPublic,
        "using GeneratedMessageMarker = void;",
    );

    CodegenPipeline::run(config)
}
```

## Validation and transformation pipeline

Before code generation, `dbc-codegen2` computes bit positions, signal usage information, and inferred signal types. It then validates the IR.

Current validation includes checks for:

- `[0|0]` physical ranges, unless `--allow-unrestricted-ranges` is used
- duplicate message IDs
- invalid signal layouts
- overlapping message bits and unused bits
- unsupported multiplexing forms
- invalid enum variants
- physical ranges that cannot be represented by the raw signal encoding
- unsafe signal scaling arithmetic

After validation, the IR is normalized for code generation by sanitizing enum variant names, deduplicating signal value enums, attaching enum types to signals, and sanitizing message, enum, and signal names.

## Name mangling rules

DBC names are not always valid Rust or C++ identifiers. `dbc-codegen2` keeps the raw DBC name in the IR, then applies deterministic mangling before code generation.

The rules below describe the current implementation.

### Identifier validity

An identifier is considered valid when:

- it is not empty
- the first character is `_` or an ASCII letter
- all following characters are `_` or ASCII alphanumeric characters
- it is not a Rust keyword
- it is not the internally reserved name `_other`

If a rendered identifier is invalid, the sanitizer prefixes it with `x`. This repeats if needed by later transformations.

Examples:

| Raw name | Reason | Sanitized base |
| --- | --- | --- |
| `1st_signal` | starts with a digit | `x1st_signal` |
| `type` | Rust keyword | `xtype` |
| `_Other` | conflicts with generated enum fallback name | `x_Other` |

After validity fixes, generated Rust names are converted with `heck` casing helpers, usually to `UpperCamelCase` for types and `snake_case` for functions/fields.

### Message names

Every message gets a `_MSG` postfix before casing. If multiple messages have the same message name case-insensitively, later duplicates get `_MSG1`, `_MSG2`, and so on.

Generated message structs use `UpperCamelCase`.

The top-level Rust message enum variant uses the message name with only the numeric part of the postfix kept. This keeps the enum variant less redundant while still disambiguating duplicates.

Examples:

| DBC message name | Duplicate index | Struct name | Rust message enum variant |
| --- | ---: | --- | --- |
| `EngineStatus` | 0 | `EngineStatusMsg` | `EngineStatus` |
| `EngineStatus` | 1 | `EngineStatusMsg1` | `EngineStatus1` |
| `123Status` | 0 | `X123StatusMsg` | `X123Status` |

### Signal names

Signal names are made unique inside each message. The first signal keeps its base name. Later signals with the same raw name case-insensitively receive numeric postfixes: `1`, `2`, and so on.

Generated getter names use `snake_case`; setter names use `set_<snake_case>`.

Examples:

| DBC signal names in one message | Generated getter names | Generated setter names |
| --- | --- | --- |
| `VehicleSpeed` | `vehicle_speed()` | `set_vehicle_speed(...)` |
| `Speed`, `speed`, `SPEED` | `speed()`, `speed1()`, `speed2()` | `set_speed(...)`, `set_speed1(...)`, `set_speed2(...)` |
| `type` | `xtype()` | `set_xtype(...)` |
| `1stValue` | `x1st_value()` | `set_x1st_value(...)` |

### Signal value enum names

Signal value enum names receive an `_ENUM` postfix. Duplicate enum names, compared case-insensitively after adding `_ENUM`, receive `_ENUM1`, `_ENUM2`, and so on.

Generated enum type names use `UpperCamelCase`.

Examples:

| DBC enum/signal enum name | Duplicate index | Generated enum type |
| --- | ---: | --- |
| `Gear` | 0 | `GearEnum` |
| `Gear` | 1 | `GearEnum1` |
| `type` | 0 | `XtypeEnum` |

### Signal value enum variant names

Enum variant names are sanitized before enum-name sanitization:

1. the raw signal name is removed from the value-description text
2. the result is converted to `UpperCamelCase`
3. if the result is empty, the variant becomes `V<raw_value>`
4. if the result is not a valid identifier, it is prefixed with `V`
5. if multiple variants still have the same name, each duplicate gets its raw numeric value appended

Examples for a signal named `Gear`:

| Raw value description | Raw value | Generated variant |
| --- | ---: | --- |
| `Gear Off` | 0 | `Off` |
| `Gear Drive` | 1 | `Drive` |
| `Gear` | 2 | `V2` |
| `1st` | 3 | `V1st` |
| `State` and another `State` | 4, 5 | `State4`, `State5` |

When `--no-enum-other` is not used, generated Rust enums include an `_Other(<repr>)` fallback variant. Because `_other` is reserved internally, a DBC variant that would become `_Other` is prefixed during validity checks.

### Multiplexed message names

For a multiplexed message named `<MessageName>`, generated Rust names follow this shape:

| Generated item | Name pattern | Example for `Sensor` |
| --- | --- | --- |
| wrapper message struct | `<MessageName>Msg` | `SensorMsg` |
| mux enum | `<MessageStructName>Mux` | `SensorMsgMux` |
| mux enum variant | `V<mux_value>` | `V0`, `V1` |
| mux variant struct | `<MessageStructName>Mux<mux_value>` | `SensorMsgMux0` |
| wrapper mux getter | `mux()` | `mux()` |
| wrapper mux setter | `set_mux<mux_value>(...)` | `set_mux0(...)` |

Generated C++ multiplexed names follow the same type-name shape, except C++ mux setters use `set_mux_<mux_value>(...)`.
