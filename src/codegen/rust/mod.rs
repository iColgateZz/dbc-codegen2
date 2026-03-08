use crate::ir::helpers::ToUpperCamelCase;
use crate::ir::message::MessageId;
use crate::ir::signal::Signal;
use crate::{codegen::Generator, ir::message::Message};
use std::collections::HashSet;

pub mod helpers;
pub use helpers::format_float;

pub mod sample_output;
// pub mod test;

pub struct RustGen {
    buf: String,
    used_can_types: HashSet<&'static str>,
}

impl Generator for RustGen {
    fn buf_mut(&mut self) -> &mut String {
        &mut self.buf
    }
}

impl RustGen {
    pub fn new() -> RustGen {
        RustGen {
            buf: String::new(),
            used_can_types: HashSet::new(),
        }
    }

    pub fn generate(mut self, messages: &[Message]) -> String {
        self.write_newline();

        self.error_enum(0);
        self.write_newline();

        self.msg_trait(0);
        self.write_newline();

        self.msg(messages, 0);
        self.write_newline();

        for msg in messages {
            self.message(msg, 0);
            self.write_newline();
        }

        self.xuse();

        self.buf
    }

    fn xuse(&mut self) {
        let mut types: Vec<&str> = vec!["Frame", "Id"];
        types.extend(&self.used_can_types);
        types.sort();
        let use_stmt = format!("use embedded_can::{{{}}};", types.join(", "));
        self.prepend_line(&use_stmt);
    }

    fn error_enum(&mut self, indent: usize) {
        self.derive(indent);
        self.write_line(indent, "pub enum CanError {");
        self.write_line(indent + 4, "Err1,");
        self.write_line(indent + 4, "Err2,");
        self.write_line(indent, "}");
    }

    fn msg_trait(&mut self, indent: usize) {
        self.write_line(indent, "pub trait CanMessage<const LEN: usize>: Sized {");
        self.write_line(
            indent + 4,
            "fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError>;",
        );
        self.write_line(indent + 4, "fn encode(&self) -> (Id, [u8; LEN]);");
        self.write_line(indent, "}");
    }

    fn msg(&mut self, messages: &[Message], indent: usize) {
        self.msg_enum(messages, indent);
        self.write_newline();

        self.msg_impl(messages, indent);
    }

    fn msg_enum(&mut self, messages: &[Message], indent: usize) {
        self.derive(indent);
        self.write_line(indent, "pub enum Msg {");

        for msg in messages {
            let s = format!("{}({})", msg.name.0, msg.name.0);
            self.write_line(indent + 4, &format!("{s},"));
        }

        self.write_line(indent, "}");
    }

    fn msg_impl(&mut self, messages: &[Message], indent: usize) {
        self.write_line(indent, "impl Msg {");

        self.write_line(
            indent + 4,
            "fn try_from(frame: &impl Frame) -> Result<Self, CanError> {",
        );

        self.write_line(indent + 8, "let id = match frame.id() {");
        self.write_line(indent + 12, "Id::Standard(sid) => sid.as_raw() as u32,");
        self.write_line(indent + 12, "Id::Extended(eid) => eid.as_raw(),");
        self.write_line(indent + 8, "};");
        self.write_newline();

        self.write_line(indent + 8, "let result = match id {");
        for msg in messages {
            let s = format!(
                "{0}::ID => Msg::{0}({0}::try_from_frame(frame)?)",
                msg.name.0
            );
            self.write_line(indent + 12, &format!("{s},"));
        }
        self.write_line(indent + 12, "_ => return Err(CanError::Err1),");
        self.write_line(indent + 8, "};");
        self.write_newline();

        self.write_line(indent + 8, "Ok(result)");

        // close try_from block
        self.write_line(indent + 4, "}");
        // close impl block
        self.write_line(indent, "}");
    }

    fn message(&mut self, msg: &Message, indent: usize) {
        for signal in &msg.signals {
            if !signal.value_descriptions.is_empty() {
                self.signal_value_enum(signal, indent);
            }
        }

        // #[derive(...)]
        self.derive(indent);

        // pub struct Name { ... }
        self.xstruct(msg, indent);
        self.write_newline();

        // impl Name { ... }
        self.ximpl(msg, indent);
        self.write_newline();

        self.impl_can_msg(msg, indent);
    }

    fn signal_value_enum(&mut self, signal: &Signal, indent: usize) {
        let signal_name = &signal.name.0.0.to_upper_camelcase();

        self.write_line(indent, "#[derive(Debug, Clone, Copy, PartialEq, Eq)]");
        self.write_line(indent, &format!("pub enum {} {{", signal_name));
        for value_desc in &signal.value_descriptions {
            self.write_line(indent + 4, &format!("{},", value_desc.description));
        }
        self.write_line(indent + 4, "_Other(u8),");
        self.write_line(indent, "}");
        self.write_newline();

        let arms_from: Vec<(String, String)> = signal
            .value_descriptions
            .iter()
            .map(|vd| (vd.value.to_string(), format!("Self::{}", vd.description)))
            .collect();
        self.write_from_impl(
            indent,
            "u8",
            signal_name,
            &arms_from,
            "_ => Self::_Other(val),",
        );

        let arms_into: Vec<(String, String)> = signal
            .value_descriptions
            .iter()
            .map(|vd| {
                (
                    format!("{}::{}", signal_name, vd.description),
                    vd.value.to_string(),
                )
            })
            .collect();
        let fallback_into = format!("{}::_Other(v) => v,", signal_name);
        self.write_from_impl(indent, signal_name, "u8", &arms_into, &fallback_into);
    }

    fn write_match_block(&mut self, indent: usize, arms: &[(String, String)], fallback: &str) {
        self.write_line(indent, "match val {");
        for (pattern, body) in arms {
            self.write_line(indent + 4, &format!("{} => {},", pattern, body));
        }
        self.write_line(indent + 4, fallback);
        self.write_line(indent, "}");
    }

    fn write_from_impl(
        &mut self,
        indent: usize,
        from_type: &str,
        to_type: &str,
        arms: &[(String, String)],
        fallback: &str,
    ) {
        self.write_line(
            indent,
            &format!("impl From<{}> for {} {{", from_type, to_type),
        );
        self.write_line(
            indent + 4,
            &format!("fn from(val: {}) -> {} {{", from_type, to_type),
        );
        self.write_match_block(indent + 8, arms, fallback);
        self.write_line(indent + 4, "}");
        self.write_line(indent, "}");
        self.write_newline();
    }

    fn derive(&mut self, indent: usize) {
        self.write_line(indent, "#[derive(Debug, Clone)]");
    }

    fn xstruct(&mut self, msg: &Message, indent: usize) {
        // pub struct Name {
        self.write_line(indent, &format!("pub struct {} {{", &msg.name.0));

        for sig in &msg.signals {
            let field = &sig.name.0.0;
            self.write_line(indent + 4, &format!("pub {}: f64,", field));
        }

        // close struct block
        self.write_line(indent, "}");
    }

    fn ximpl(&mut self, msg: &Message, indent: usize) {
        // impl Name {
        self.write_line(indent, &format!("impl {} {{", &msg.name.0));

        // const ID
        self.id(msg, indent);

        // const LEN
        self.len(msg, indent);

        // close impl block
        self.write_line(indent, "}");
    }

    fn id(&mut self, msg: &Message, indent: usize) {
        let id = match msg.id {
            MessageId::Standard(id) => id as u32,
            MessageId::Extended(id) => id,
        };

        self.write_line(indent + 4, &format!("pub const ID: u32 = {};", id));
    }

    fn len(&mut self, msg: &Message, indent: usize) {
        self.write_line(indent + 4, &format!("pub const LEN: usize = {};", msg.size));
    }

    fn impl_can_msg(&mut self, msg: &Message, indent: usize) {
        self.write_line(
            indent,
            &format!("impl CanMessage<{{ {0}::LEN }}> for {0} {{", msg.name.0),
        );
        self.try_from_frame(msg, indent + 4);
        self.write_newline();

        self.encode(msg, indent + 4);

        // close impl block
        self.write_line(indent, "}");
    }

    fn try_from_frame(&mut self, msg: &Message, indent: usize) {
        self.write_line(
            indent,
            "fn try_from_frame(frame: &impl Frame) -> Result<Self, CanError> {",
        );
        self.write_line(indent + 4, "let data = frame.data();");
        self.write_newline();

        let mut byte = 0usize;
        for sig in &msg.signals {
            let raw = format!("raw_{}", sig.name.0.0);
            self.write_line(
                indent + 4,
                &format!(
                    "let {} = u16::from_le_bytes([data[{}], data[{}]]);",
                    raw,
                    byte,
                    byte + 1
                ),
            );
            byte += 2;
        }
        self.write_newline();

        self.write_line(indent + 4, "Ok(Self {");
        for sig in &msg.signals {
            let name = &sig.name.0.0;
            let raw = format!("raw_{}", name);
            self.write_line(
                indent + 8,
                &format!("{}: {} as f64 * {},", name, raw, format_float(sig.factor)),
            );
        }
        // close Ok block
        self.write_line(indent + 4, "})");
        // close try_from_frame block
        self.write_line(indent, "}");
    }

    fn encode(&mut self, msg: &Message, indent: usize) {
        self.write_line(
            indent,
            &format!("fn encode(&self) -> (Id, [u8; {}::LEN]) {{", msg.name.0),
        );
        self.write_line(
            indent + 4,
            &format!("let mut data = [0u8; {}::LEN];", msg.name.0),
        );
        self.write_newline();

        let mut byte = 0usize;
        for sig in &msg.signals {
            let name = &sig.name.0.0;
            let raw = format!("raw_{}", name);

            self.write_line(
                indent + 4,
                &format!(
                    "let {} = (self.{} / {}) as u16;",
                    raw,
                    name,
                    format_float(sig.factor)
                ),
            );

            self.write_line(
                indent + 4,
                &format!("let {}_bytes = {}.to_le_bytes();", name, raw),
            );
            self.write_line(indent + 4, &format!("data[{}] = {}_bytes[0];", byte, name));
            self.write_line(
                indent + 4,
                &format!("data[{}] = {}_bytes[1];", byte + 1, name),
            );
            self.write_newline();

            byte += 2;
        }

        let id = match msg.id {
            MessageId::Standard(_) => {
                self.used_can_types.insert("StandardId");
                "Id::Standard(StandardId::new(Self::ID as u16).unwrap())"
            }
            MessageId::Extended(_) => {
                self.used_can_types.insert("ExtendedId");
                "Id::Extended(ExtendedId::new(Self::ID).unwrap())"
            }
        };
        self.write_line(indent + 4, &format!("let id = {id};"));
        self.write_line(indent + 4, "(id, data)");

        // close encode block
        self.write_line(indent, "}");
    }
}
