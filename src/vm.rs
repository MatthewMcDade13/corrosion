use anyhow::bail;
use log::debug;

use crate::{
    ast::AstWalkError,
    compiler::{compile_source, Chunk},
    value::{Object, Value},
};

macro_rules! binary_op {
    ($vm:ident, $op:tt, $op_return:expr) => {
        let b = $vm.pop()?.as_number()?;
        let a = $vm.pop()?.as_number()?;
        $vm.push($op_return(a $op b));
    };
}

pub struct VM {
    pc: usize,
    chunk: Chunk,
    stack: Vec<Value>,
}

impl VM {
    const STACK_SIZE: usize = 256;
    pub fn new() -> Self {
        Self {
            pc: 0,
            chunk: Chunk::new(),
            stack: Vec::with_capacity(Self::STACK_SIZE),
        }
    }

    pub fn reset(&mut self, chunk: Chunk) {
        self.pc = 0;
        self.chunk = chunk;
        self.stack.clear();
    }

    pub fn stack_top(&self) -> Option<&Value> {
        self.stack.last()
    }

    pub fn push(&mut self, value: Value) {
        self.stack.push(value)
    }

    pub fn pop(&mut self) -> anyhow::Result<Value> {
        match self.stack.pop() {
            Some(v) => Ok(v),
            None => bail!("Stack is empty, nothing to pop"),
        }
    }

    pub fn interpret(&mut self, ops: &[Opcode]) -> anyhow::Result<()> {
        todo!()
    }

    pub fn next_op(&mut self) -> Opcode {
        let op = self.chunk.opcode_at(self.pc);
        self.pc += 1;
        op
    }

    pub fn interpret_source(&mut self, source: &str) -> anyhow::Result<()> {
        let chunk = compile_source(source)?;
        self.reset(chunk);
        self.run()
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        while self.pc < self.chunk.instructions_len() {
            let op = self.next_op();
            match op.ty() {
                OpcodeType::Return => {
                    println!("{}", self.pop()?);
                    return Ok(());
                }
                OpcodeType::Constant => {
                    let cindex = self.next_op();
                    let c = self.chunk.constant_at(cindex.0).clone();
                    self.push(c);
                }
                OpcodeType::Negate => {
                    let iback = self.stack.len() - 1;
                    let val = &self.stack[iback];
                    if let Value::Number(n) = val {
                        self.stack[iback] = Value::Number(-n);
                    } else {
                        bail!("Cannot negate non-number at top of stack: {:?}", val)
                    }
                }
                OpcodeType::Add => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match a {
                        Value::Number(ln) => {
                            if let Value::Number(rn) = b {
                                self.push(Value::Number(ln + rn));
                            } else {
                                bail!("Addition operands must be 2 numbers or 2 strings.");
                            }
                        }
                        Value::Obj(lobj) => match lobj {
                            Object::String(lstr) => {
                                if let Value::Obj(Object::String(rstr)) = b {
                                    self.push(Value::Obj(Object::String(lstr + &rstr)))
                                } else {
                                    bail!("Addition operands must be 2 numbers or 2 strings.");
                                }
                            }
                        },
                        _ => {
                            bail!("Addition operands must be 2 numbers or 2 strings.");
                        }
                    }
                }
                OpcodeType::Subtract => {
                    binary_op!(self, -, Value::Number);
                }
                OpcodeType::Mult => {
                    binary_op!(self, *, Value::Number);
                }
                OpcodeType::Div => {
                    binary_op!(self, /, Value::Number);
                }
                OpcodeType::Nil => {
                    self.push(Value::Nil);
                }
                OpcodeType::False => self.push(Value::Boolean(false)),
                OpcodeType::True => self.push(Value::Boolean(true)),
                OpcodeType::Not => {
                    let iback = self.stack.len() - 1;
                    let val = &self.stack[iback];
                    self.stack[iback] = Value::Boolean(val.is_falsey());
                }
                OpcodeType::Equal => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Boolean(a == b));
                }
                OpcodeType::GreaterThan => {
                    binary_op!(self, >, Value::Boolean);
                }
                OpcodeType::LessThan => {
                    binary_op!(self, <, Value::Boolean);
                }
                OpcodeType::Unknown => {
                    bail!("Unknown opcode encountered: {:X}", op.0)
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Opcode(pub usize);

impl Opcode {
    pub const fn ty(&self) -> OpcodeType {
        match *self {
            Self(0) => OpcodeType::Return,
            Self(1) => OpcodeType::Constant,
            Self(2) => OpcodeType::Negate,
            Self(3) => OpcodeType::Add,
            Self(4) => OpcodeType::Subtract,
            Self(5) => OpcodeType::Mult,
            Self(6) => OpcodeType::Div,
            Self(7) => OpcodeType::Nil,
            Self(8) => OpcodeType::True,
            Self(9) => OpcodeType::False,
            Self(10) => OpcodeType::Not,
            Self(11) => OpcodeType::Equal,
            Self(12) => OpcodeType::GreaterThan,
            Self(13) => OpcodeType::LessThan,
            _ => OpcodeType::Unknown,
        }
    }
}

impl From<OpcodeType> for Opcode {
    fn from(value: OpcodeType) -> Self {
        Opcode(value as usize)
    }
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X} => Opcode::{:?}", self.0, self.ty())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OpcodeType {
    Return = 0,
    Constant,
    Negate,
    Add,
    Subtract,
    Mult,
    Div,
    Nil,
    True,
    False,
    Not,
    Equal,
    GreaterThan,
    LessThan,
    Unknown,
}
