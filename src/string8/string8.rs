use std::ffi::CStr;
use std::os::raw::c_char;

use crate::{ArenaScope, ArenaVec};

#[derive(Debug)]
pub struct String8<'a> {
    bytes: ArenaVec<'a, u8>,
}

impl<'a> String8<'a> {
    pub fn new_in(scope: &'a ArenaScope<'a>) -> Self {
        let mut bytes = ArenaVec::new_in(scope);
        bytes.push(0);
        Self { bytes }
    }

    pub fn with_capacity_in(capacity: usize, scope: &'a ArenaScope<'a>) -> Self {
        let mut bytes = ArenaVec::with_capacity_in(capacity.saturating_add(1), scope);
        bytes.push(0);
        Self { bytes }
    }

    pub fn from_bytes_in(bytes: &[u8], scope: &'a ArenaScope<'a>) -> Self {
        let mut string = Self::with_capacity_in(bytes.len(), scope);
        string.append_bytes(bytes);
        string
    }

    pub fn from_str_in(value: &str, scope: &'a ArenaScope<'a>) -> Self {
        Self::from_bytes_in(value.as_bytes(), scope)
    }

    pub fn len(&self) -> usize {
        self.bytes.len().saturating_sub(1)
    }

    pub fn capacity(&self) -> usize {
        self.bytes.capacity().saturating_sub(1)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.bytes.clear();
        self.bytes.push(0);
    }

    pub fn append_byte(&mut self, byte: u8) {
        let len = self.len();
        self.bytes[len] = byte;
        self.bytes.push(0);
    }

    pub fn append_bytes(&mut self, src: &[u8]) {
        if src.is_empty() {
            return;
        }

        let len = self.len();
        self.bytes[len] = src[0];
        for &byte in &src[1..] {
            self.bytes.push(byte);
        }
        self.bytes.push(0);
    }

    pub fn append_str(&mut self, src: &str) {
        self.append_bytes(src.as_bytes());
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes.as_slice()[..self.len()]
    }

    pub fn as_bytes_with_nul(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    pub fn as_c_str(&self) -> &CStr {
        CStr::from_bytes_with_nul(self.as_bytes_with_nul()).expect("String8 contains interior nul")
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.bytes.as_slice().as_ptr()
    }

    pub fn as_c_ptr(&self) -> *const c_char {
        self.as_ptr().cast::<c_char>()
    }
}
