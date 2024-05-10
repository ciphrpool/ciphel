use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use num_traits::ToBytes;

use crate::{
    semantic::{AccessLevel, MutRc},
    vm::vm::RuntimeError,
};

pub const STACK_SIZE: usize = 2024;

#[derive(Debug, Clone)]
pub enum StackError {
    StackOverflow,
    StackUnderflow,
    ReadError,
    WriteError,
    Default,
}

impl Into<RuntimeError> for StackError {
    fn into(self) -> RuntimeError {
        RuntimeError::StackError(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Offset {
    SB(usize),
    ST(isize),
    FB(usize),
    FZ(isize),
    FP(usize),
    FE(usize, usize),
}

impl Offset {
    pub fn name(&self, level: &AccessLevel) -> String {
        match self {
            Offset::SB(n) => format!("SB[{n}{}]", level.name()),
            Offset::ST(n) => format!("ST[{n}{}]", level.name()),
            Offset::FB(n) => format!("FB[{n}{}]", level.name()),
            Offset::FZ(n) => format!("FZ[{n}{}]", level.name()),
            Offset::FP(n) => format!("FP[{n}{}]", level.name()),
            Offset::FE(n, m) => format!("FE[{n},{m}{}]", level.name()),
        }
    }
}

impl Default for Offset {
    fn default() -> Self {
        Offset::ST(0)
    }
}

#[derive(Debug, Clone)]
pub struct Stack {
    stack: [u8; STACK_SIZE],
    pub registers: Registers,
    // top: Cell<usize>,
    // registers.bottom: Cell<usize>,
    // registers.zero: Cell<usize>,
}

#[derive(Debug, Clone)]
pub struct Registers {
    pub top: Cell<usize>,
    pub bottom: Cell<usize>,
    pub zero: Cell<usize>,
    pub params_start: Cell<usize>,
    pub link: Cell<usize>,
    pub window: Cell<usize>,
    pub r1: Cell<u64>,
    pub r2: Cell<u64>,
    pub r3: Cell<u64>,
    pub r4: Cell<u64>,
}

#[derive(Debug, Clone, Copy)]
pub enum UReg {
    R1,
    R2,
    R3,
    R4,
}

impl UReg {
    pub fn name(&self) -> &'static str {
        match self {
            UReg::R1 => "rg1",
            UReg::R2 => "rg2",
            UReg::R3 => "rg3",
            UReg::R4 => "rg4",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Frame {
    bottom: usize,
    zero: usize,
    params_start: usize,
    link: usize,
}
impl Into<Frame> for &Registers {
    fn into(self) -> Frame {
        Frame {
            bottom: self.bottom.get(),
            zero: self.zero.get(),
            params_start: self.params_start.get(),
            link: self.link.get(),
        }
    }
}
impl Frame {
    fn from(frame: Self, buffer: &[u8]) -> Result<Self, StackError> {
        // Retrieve previous link
        let data = TryInto::<[u8; 8]>::try_into(
            &buffer[frame.params_start
                - (Registers::link_size()
                    + Registers::bottom_size()
                    + Registers::zero_size()
                    + Registers::params_start_size())
                ..frame.params_start
                    - (Registers::bottom_size()
                        + Registers::zero_size()
                        + Registers::params_start_size())],
        )
        .map_err(|_| StackError::ReadError)?;
        let link = u64::from_le_bytes(data);

        // Retrieve previous bottom
        let data = TryInto::<[u8; 8]>::try_into(
            &buffer[frame.params_start
                - (Registers::bottom_size()
                    + Registers::zero_size()
                    + Registers::params_start_size())
                ..frame.params_start - (Registers::zero_size() + Registers::params_start_size())],
        )
        .map_err(|_| StackError::ReadError)?;
        let bottom = u64::from_le_bytes(data);

        // Retrieve previous params start
        let data = TryInto::<[u8; 8]>::try_into(
            &buffer[frame.params_start - (Registers::zero_size() + Registers::params_start_size())
                ..frame.params_start - (Registers::zero_size())],
        )
        .map_err(|_| StackError::ReadError)?;
        let params_start = u64::from_le_bytes(data);

        // Retrieve previous zero
        let data = TryInto::<[u8; 8]>::try_into(
            &buffer[frame.params_start - (Registers::zero_size())..frame.params_start],
        )
        .map_err(|_| StackError::ReadError)?;
        let zero = u64::from_le_bytes(data);
        Ok(Self {
            bottom: bottom as usize,
            zero: zero as usize,
            params_start: params_start as usize,
            link: link as usize,
        })
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            top: Cell::new(0),
            bottom: Cell::new(0),
            zero: Cell::new(0),
            params_start: Cell::new(0),
            link: Cell::new(0),
            window: Cell::new(0),
            r1: Cell::new(0),
            r2: Cell::new(0),
            r3: Cell::new(0),
            r4: Cell::new(0),
        }
    }
}
impl Registers {
    const fn bottom_size() -> usize {
        8
    }
    const fn zero_size() -> usize {
        8
    }
    const fn link_size() -> usize {
        8
    }
    const fn params_start_size() -> usize {
        8
    }
}
// #[derive(Debug, Clone)]
// pub struct Frame {
//     zero: usize,
//     bottom: usize,
// }

#[derive(Debug, Clone)]
pub struct StackSlice {
    pub offset: Offset,
    pub size: usize,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            stack: [0; STACK_SIZE],
            registers: Registers::default(),
        }
    }

    pub fn open_window(&mut self) -> Result<(), StackError> {
        let bottom = self.registers.top.get();
        let _ = self.push_with(&(self.registers.window.get() as u64).to_le_bytes())?;
        self.registers.window.set(bottom);
        Ok(())
    }

    pub fn close_window(&mut self) -> Result<(), StackError> {
        let bottom = self.registers.window.get();

        let previous_windows = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&self.stack[bottom..bottom + 8])
                .map_err(|_| StackError::ReadError)?,
        );
        self.registers.window.set(previous_windows as usize);
        self.registers.top.set(bottom);
        Ok(())
    }

    pub fn frame(&mut self, params_size: usize, link: usize) -> Result<(), StackError> {
        let bottom = self.registers.top.get();

        let frame_meta_size = Registers::link_size()
            + Registers::bottom_size()
            + Registers::zero_size()
            + Registers::params_start_size()
            + params_size;
        let _ = self.push_with_zero(frame_meta_size)?;

        // Copy past link
        self.stack[bottom..bottom + Registers::link_size()]
            .copy_from_slice(&(self.registers.link.get() as u64).to_le_bytes());
        // Copy past FB
        self.stack[bottom + Registers::link_size()
            ..bottom + Registers::link_size() + Registers::bottom_size()]
            .copy_from_slice(&(self.registers.bottom.get() as u64).to_le_bytes());
        // Copy past FParamStart
        self.stack[bottom + Registers::link_size() + Registers::bottom_size()
            ..bottom
                + Registers::link_size()
                + Registers::bottom_size()
                + Registers::params_start_size()]
            .copy_from_slice(&(self.registers.params_start.get() as u64).to_le_bytes());
        // Copy past FZ
        self.stack[bottom
            + Registers::link_size()
            + Registers::bottom_size()
            + Registers::params_start_size()
            ..bottom
                + Registers::link_size()
                + Registers::bottom_size()
                + Registers::params_start_size()
                + Registers::zero_size()]
            .copy_from_slice(&(self.registers.zero.get() as u64).to_le_bytes());

        // Update FB
        self.registers.bottom.set(bottom);
        // Update FZ
        self.registers.zero.set(
            bottom
                + Registers::link_size()
                + Registers::bottom_size()
                + Registers::params_start_size()
                + Registers::zero_size()
                + params_size,
        );
        // Update FP
        self.registers.params_start.set(
            bottom
                + Registers::link_size()
                + Registers::bottom_size()
                + Registers::params_start_size()
                + Registers::zero_size(),
        );
        // Update Link
        self.registers.link.set(link);
        Ok(())
    }

    pub fn clean(&mut self) -> Result<(), StackError> {
        let _top = self.top();

        if self.registers.bottom.get() != self.registers.params_start.get()
            && self.registers.params_start.get()
                >= self.registers.bottom.get()
                    + Registers::link_size()
                    + Registers::bottom_size()
                    + Registers::params_start_size()
                    + Registers::zero_size()
        {
            let Frame {
                bottom,
                zero,
                params_start,
                link,
            } = Frame::from((&self.registers).into(), self.stack.as_ref())?;
            // update registers
            self.registers.top.set(self.registers.bottom.get());

            self.registers.link.set(link);
            self.registers.bottom.set(bottom);
            self.registers.params_start.set(params_start);
            self.registers.zero.set(zero);
        } else {
            self.registers.top.set(self.registers.bottom.get());
        }
        Ok(())
    }

    pub fn set_reg(&self, reg: UReg, idx: u64) -> u64 {
        match reg {
            UReg::R1 => {
                let old = self.registers.r1.get();
                self.registers.r1.set(idx);
                old
            }
            UReg::R2 => {
                let old = self.registers.r2.get();
                self.registers.r2.set(idx);
                old
            }
            UReg::R3 => {
                let old = self.registers.r3.get();
                self.registers.r3.set(idx);
                old
            }
            UReg::R4 => {
                let old = self.registers.r4.get();
                self.registers.r4.set(idx);
                old
            }
        }
    }
    pub fn get_reg(&self, reg: UReg) -> u64 {
        match reg {
            UReg::R1 => self.registers.r1.get(),
            UReg::R2 => self.registers.r2.get(),
            UReg::R3 => self.registers.r3.get(),
            UReg::R4 => self.registers.r4.get(),
        }
    }
    pub fn reg_add(&self, reg: UReg, x: u64) -> Result<(), StackError> {
        match reg {
            UReg::R1 => {
                if let Some(res) = self.registers.r1.get().checked_add(x) {
                    self.registers.r1.set(res);
                    Ok(())
                } else {
                    Err(StackError::WriteError)
                }
            }
            UReg::R2 => {
                if let Some(res) = self.registers.r2.get().checked_add(x) {
                    self.registers.r2.set(res);
                    Ok(())
                } else {
                    Err(StackError::WriteError)
                }
            }
            UReg::R3 => {
                if let Some(res) = self.registers.r3.get().checked_add(x) {
                    self.registers.r3.set(res);
                    Ok(())
                } else {
                    Err(StackError::WriteError)
                }
            }
            UReg::R4 => {
                if let Some(res) = self.registers.r4.get().checked_add(x) {
                    self.registers.r4.set(res);
                    Ok(())
                } else {
                    Err(StackError::WriteError)
                }
            }
        }
    }
    pub fn reg_sub(&self, reg: UReg, x: u64) -> Result<(), StackError> {
        match reg {
            UReg::R1 => {
                if let Some(res) = self.registers.r1.get().checked_sub(x) {
                    self.registers.r1.set(res);
                }
            }
            UReg::R2 => {
                if let Some(res) = self.registers.r2.get().checked_sub(x) {
                    self.registers.r2.set(res);
                }
            }
            UReg::R3 => {
                if let Some(res) = self.registers.r3.get().checked_sub(x) {
                    self.registers.r3.set(res);
                }
            }
            UReg::R4 => {
                if let Some(res) = self.registers.r4.get().checked_sub(x) {
                    self.registers.r4.set(res);
                }
            }
        }
        Ok(())
    }

    pub fn top(&self) -> usize {
        self.registers.top.get()
    }
    pub fn push(&mut self, size: usize) -> Result<(), StackError> {
        let top = self.top();
        if top + size >= STACK_SIZE {
            return Err(StackError::StackOverflow);
        }
        self.registers.top.set(top + size);
        Ok(())
    }

    pub fn push_with(&mut self, data: &[u8]) -> Result<(), StackError> {
        let top = self.top();
        if top + data.len() >= STACK_SIZE {
            return Err(StackError::StackOverflow);
        }
        self.stack[top..top + data.len()].copy_from_slice(&data);
        self.registers.top.set(top + data.len());
        Ok(())
    }
    pub fn push_with_zero(&mut self, size: usize) -> Result<(), StackError> {
        self.push_with(&vec![0; size])
    }
    pub fn pop<'env>(&'env mut self, size: usize) -> Result<&'env [u8], StackError> {
        let top = self.top();
        if top < size {
            return Err(StackError::StackUnderflow);
        }
        let res = &self.stack[top - size..top];
        self.registers.top.set(top - size);
        Ok(res)
    }

    pub fn compute_absolute_address(
        &self,
        offset: Offset,
        level: AccessLevel,
    ) -> Result<usize, StackError> {
        let top = self.top();
        match offset {
            Offset::SB(idx) => {
                if idx >= top {
                    return Err(StackError::ReadError);
                }
                Ok(idx)
            }
            Offset::ST(idx) => {
                if idx < 0 && ((-idx) as usize > top) {
                    return Err(StackError::ReadError);
                } else if idx >= 0 && (idx as usize >= top) {
                    return Err(StackError::ReadError);
                }
                let start = if idx < 0 {
                    top - (-idx as usize)
                } else {
                    top + (idx as usize)
                };
                Ok(start)
            }
            Offset::FB(idx) => {
                let frame_bottom = match level {
                    AccessLevel::General => 0,
                    AccessLevel::Direct => self.registers.bottom.get(),
                    AccessLevel::Backward(backward) => {
                        let mut frame = (&self.registers).into();
                        let mut backward = backward;
                        while backward != 0 {
                            let previous_frame = Frame::from(frame, self.stack.as_ref())?;
                            if backward == 1 {
                                backward = 0;
                            } else {
                                backward -= 1;
                            }
                            frame = previous_frame;
                        }
                        frame.bottom
                    }
                };
                if frame_bottom + idx >= top {
                    return Err(StackError::ReadError);
                }
                Ok(frame_bottom + idx)
            }
            Offset::FZ(idx) => {
                let frame_zero = match level {
                    AccessLevel::General => 0,
                    AccessLevel::Direct => self.registers.zero.get(),
                    AccessLevel::Backward(backward) => {
                        let mut frame = (&self.registers).into();
                        let mut backward = backward;
                        while backward != 0 {
                            let previous_frame = Frame::from(frame, self.stack.as_ref())?;
                            if backward == 1 {
                                backward = 0;
                            } else {
                                backward -= 1;
                            }
                            frame = previous_frame;
                        }
                        frame.zero
                    }
                };
                // let frame_zero = self.registers.zero.get();
                let start = if idx <= 0 {
                    frame_zero - (-idx) as usize
                } else {
                    frame_zero + (idx as usize)
                };
                if start >= top {
                    return Err(StackError::ReadError);
                }
                Ok(start)
            }
            Offset::FP(idx) => {
                let frame_params_start = match level {
                    AccessLevel::General => 0,
                    AccessLevel::Direct => self.registers.params_start.get(),
                    AccessLevel::Backward(backward) => {
                        let mut frame = (&self.registers).into();
                        let mut backward = backward;
                        while backward != 0 {
                            let previous_frame = Frame::from(frame, self.stack.as_ref())?;
                            if backward == 1 {
                                backward = 0;
                            } else {
                                backward -= 1;
                            }
                            frame = previous_frame;
                        }
                        frame.params_start
                    }
                };

                if frame_params_start + idx >= top {
                    return Err(StackError::ReadError);
                }
                Ok(frame_params_start + idx)
            }
            Offset::FE(_, _) => unreachable!(),
        }
    }

    pub fn read<'env>(
        &'env self,
        offset: Offset,
        level: AccessLevel,
        size: usize,
    ) -> Result<&'env [u8], StackError> {
        let top = self.top();
        let start = self.compute_absolute_address(offset, level)?;
        if start >= top || start + size > top {
            return Err(StackError::ReadError);
        }
        Ok(&self.stack[start..start + size])
    }

    pub fn read_utf8<'env>(
        &'env self,
        address: Offset,
        level: AccessLevel,
        idx: usize,
        len: usize,
    ) -> Result<([u8; 4], usize), StackError> {
        let top = self.top();
        let address = self.compute_absolute_address(address, level)?;
        if address >= top {
            return Err(StackError::ReadError);
        }
        let mut offset = 0;
        let mut current_idx = 0;
        let mut byte = self.stack[address + offset];

        while current_idx < idx {
            byte = self.stack[address + offset];
            if offset >= len {
                return Err(StackError::ReadError);
            }
            match byte {
                // 7-bit ASCII character (U+0000 to U+007F)
                0x00..=0x7F => {
                    offset += 1;
                    current_idx += 1;
                }
                // Two-byte character (U+0080 to U+07FF)
                0xC0..=0xDF => {
                    if (address + offset) + 1 >= STACK_SIZE {
                        return Err(StackError::ReadError);
                    }
                    let in_byte = self.stack[(address + offset) + 1];
                    if (in_byte & 0xC0) != 0x80 {
                        return Err(StackError::ReadError);
                    }
                    offset += 2;
                    current_idx += 1;
                }
                // Three-byte character (U+0800 to U+FFFF)
                0xE0..=0xEF => {
                    for i in 1..3 {
                        if (address + offset) + i >= STACK_SIZE {
                            return Err(StackError::ReadError);
                        }
                        let in_byte = self.stack[(address + offset) + i];
                        if (in_byte & 0xC0) != 0x80 {
                            return Err(StackError::ReadError);
                        }
                    }
                    offset += 3;
                    current_idx += 1;
                }
                // Four-byte character (U+10000 to U+10FFFF)
                0xF0..=0xF7 => {
                    for i in 1..4 {
                        if (address + offset) + i >= STACK_SIZE {
                            return Err(StackError::ReadError);
                        }
                        let in_byte = self.stack[(address + offset) + i];
                        if (in_byte & 0xC0) != 0x80 {
                            return Err(StackError::ReadError);
                        }
                    }
                    offset += 4;
                    current_idx += 1;
                }
                _ => {
                    return Err(StackError::ReadError);
                }
            }
        }

        if current_idx != idx {
            return Err(StackError::ReadError);
        }

        byte = self.stack[address + offset];
        let mut bytes = [byte, 0u8, 0u8, 0u8];
        let mut size = 1;
        match byte {
            // 7-bit ASCII character (U+0000 to U+007F)
            0x00..=0x7F => {}
            // Two-byte character (U+0080 to U+07FF)
            0xC0..=0xDF => {
                if (address + offset) + 1 >= STACK_SIZE {
                    return Err(StackError::ReadError);
                }
                let in_byte = self.stack[(address + offset) + 1];
                if (in_byte & 0xC0) != 0x80 {
                    return Err(StackError::ReadError);
                }
                bytes[1] = in_byte;
                size = 2;
            }
            // Three-byte character (U+0800 to U+FFFF)
            0xE0..=0xEF => {
                for i in 1..3 {
                    if (address + offset) + i >= STACK_SIZE {
                        return Err(StackError::ReadError);
                    }
                    let in_byte = self.stack[(address + offset) + i];
                    if (in_byte & 0xC0) != 0x80 {
                        return Err(StackError::ReadError);
                    }
                    bytes[i] = in_byte;
                }
                size = 3;
            }
            // Four-byte character (U+10000 to U+10FFFF)
            0xF0..=0xF7 => {
                for i in 1..4 {
                    if (address + offset) + i >= STACK_SIZE {
                        return Err(StackError::ReadError);
                    }
                    let in_byte = self.stack[(address + offset) + i];
                    if (in_byte & 0xC0) != 0x80 {
                        return Err(StackError::ReadError);
                    }
                    bytes[i] = in_byte;
                }
                size = 4;
            }
            _ => {
                return Err(StackError::ReadError);
            }
        }

        Ok((bytes, offset))
    }

    // pub fn read_last(&self, size: usize) -> Result<Vec<u8>, StackError> {
    //     let top = self.top();
    //     if top < size {
    //         return Err(StackError::ReadError);
    //     }
    //     let borrowed_buffer = self.stack.borrow();
    //     Ok(borrowed_buffer[top - size..top].to_vec())
    // }

    pub fn write(
        &mut self,
        offset: Offset,
        level: AccessLevel,
        data: &[u8],
    ) -> Result<(), StackError> {
        let top = self.top();
        let size = data.len();
        let start = self.compute_absolute_address(offset, level)?;
        if start >= top || start + size > top {
            return Err(StackError::WriteError);
        }
        self.stack[start..start + size].copy_from_slice(&data);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_push() {
        let mut stack = Stack::new();
        let _ = stack.push(8).expect("Push should have succeeded");
        assert_eq!(stack.top(), 8);
    }

    #[test]
    fn robustness_push() {
        let mut stack = Stack::new();
        let _ = stack
            .push(STACK_SIZE + 1)
            .expect_err("Push should have failed");
    }

    #[test]
    fn valid_pop() {
        let mut stack = Stack::new();
        let _ = stack.push(64).expect("Push should have succeeded");
        let _ = stack.pop(32).expect("Pop should have succeeded");

        assert_eq!(stack.top(), 32);
    }

    #[test]
    fn robustness_pop() {
        let mut stack = Stack::new();
        let _ = stack.pop(32).expect_err("Pop should have failed");
    }

    #[test]
    fn valid_read() {
        let mut stack = Stack::new();
        let _ = stack.push(8).expect("Push should have succeeded");

        stack.stack[0..8].copy_from_slice(&[1u8; 8]);

        let data = stack
            .read(Offset::SB(0), AccessLevel::Direct, 8)
            .expect("Read should have succeeded");
        assert_eq!(data, vec![1; 8]);
    }

    #[test]
    fn robustness_read() {
        let stack = Stack::new();
        let _ = stack
            .read(Offset::SB(0), AccessLevel::Direct, 8)
            .expect_err("Read should have failed");
    }

    #[test]
    fn valid_write() {
        let mut stack = Stack::new();
        let _ = stack.push(8).expect("Push should have succeeded");

        let _ = stack
            .write(Offset::SB(0), AccessLevel::Direct, &vec![1; 8])
            .expect("Write should have succeeded");

        assert_eq!(stack.stack[0..8], vec![1; 8]);
    }

    #[test]
    fn robustness_write() {
        let mut stack = Stack::new();
        let _ = stack
            .write(Offset::SB(0), AccessLevel::Direct, &vec![1; 8])
            .expect_err("Read should have failed");
    }

    #[test]
    fn valid_frame() {
        let mut stack = Stack::new();
        let _ = stack.push(8).expect("Push should have succeeded");

        let _ = stack
            .frame(0, 0)
            .expect("Frame creation should have succeeded");
        let _ = stack.push(8).expect("Push should have succeeded");
        assert_eq!(stack.registers.bottom.get(), 8);
        assert_eq!(stack.registers.zero.get(), 40);

        assert_eq!(stack.top(), 48);
    }

    #[test]
    fn valid_frame_clean() {
        let mut stack = Stack::new();
        let _ = stack
            .frame(0, 0)
            .expect("Frame creation should have succeeded");
        let _ = stack.push(8).expect("Push should have succeeded");
        let _ = stack.clean().expect("Clean should have succeeded");

        assert_eq!(stack.top(), 0);
    }
}
