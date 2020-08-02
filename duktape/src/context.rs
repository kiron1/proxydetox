use crate::Stack;
use duktape_sys::{duk_context, duk_destroy_heap, duke_create_heap_default};
use std::fmt::{Error, Formatter};
use std::ptr::null_mut;
use std::result::Result;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CreateContextError;

impl std::error::Error for CreateContextError {}

impl std::fmt::Display for CreateContextError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "create contex error")
    }
}

pub struct Context {
    ptr: *mut duk_context,
}

impl Context {
    pub fn new() -> Result<Self, CreateContextError> {
        let ptr = unsafe { duke_create_heap_default() };
        if ptr == null_mut() {
            Err(CreateContextError)
        } else {
            Ok(Context { ptr })
        }
    }
}

impl Stack for Context {
    unsafe fn ptr(&mut self) -> *mut duk_context {
        self.ptr
    }
}
unsafe impl Send for Context {}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            duk_destroy_heap(self.ptr);
        }
    }
}

pub struct ContextRef {
    ptr: *mut duk_context,
}

impl Stack for ContextRef {
    unsafe fn ptr(&mut self) -> *mut duk_context {
        self.ptr
    }
}

unsafe impl Send for ContextRef {}

impl From<*mut duk_context> for ContextRef {
    fn from(ptr: *mut duk_context) -> ContextRef {
        ContextRef { ptr }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let mut ctx = Context::new().unwrap();
        match ctx.eval("5+5") {
            Ok(crate::Value::Number(k)) => assert_eq!(k, 10.0f64),
            _ => assert!(false),
        }
    }
}
