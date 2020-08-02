use duktape_sys::{
    duk_call, duk_context, duk_eval_string, duk_get_boolean, duk_get_global_string, duk_get_number,
    duk_get_string, duk_get_type, duk_pop, duk_push_boolean, duk_push_c_function, duk_push_null,
    duk_push_string, duk_put_global_string, duk_require_stack, duk_ret_t, DUK_TYPE_BOOLEAN,
    DUK_TYPE_NONE, DUK_TYPE_NULL, DUK_TYPE_NUMBER, DUK_TYPE_STRING, DUK_TYPE_UNDEFINED,
};
use std::ffi::{CStr, CString};

use crate::error::TypeError;
use std::result::Result;

pub trait Stack {
    unsafe fn ptr(&mut self) -> *mut duk_context;

    fn pop_string(&mut self) -> Result<crate::Value, TypeError> {
        let result = self.get_string(-1)?;

        unsafe {
            duk_pop(self.ptr());
        }
        Ok(crate::Value::String(result))
    }

    fn pop(&mut self) -> Result<crate::Value, TypeError> {
        let typeid: u32 = unsafe { duk_get_type(self.ptr(), -1) as u32 };
        match typeid {
            DUK_TYPE_NONE => Err(TypeError::NoneType),
            DUK_TYPE_UNDEFINED => {
                let r = Ok(crate::Value::Undefined);
                unsafe {
                    duk_pop(self.ptr());
                }
                r
            }
            DUK_TYPE_NULL => {
                let r = Ok(crate::Value::Null);
                unsafe {
                    duk_pop(self.ptr());
                }
                r
            }
            DUK_TYPE_BOOLEAN => {
                let r = Ok(crate::Value::from(
                    unsafe { duk_get_boolean(self.ptr(), -1) } != 0,
                ));
                unsafe {
                    duk_pop(self.ptr());
                }
                r
            }
            DUK_TYPE_NUMBER => {
                let r = Ok(crate::Value::from(unsafe {
                    duk_get_number(self.ptr(), -1)
                }));
                unsafe {
                    duk_pop(self.ptr());
                }
                r
            }
            DUK_TYPE_STRING => self.pop_string(),
            _ => Err(TypeError::NoneType),
        }
    }

    fn eval(&mut self, src: &str) -> Result<crate::Value, TypeError> {
        unsafe {
            duk_eval_string(
                self.ptr(),
                &CString::new(src).map_err(|_| TypeError::BadString)?,
            );
        }
        self.pop()
    }

    fn get_global_string(&mut self, name: &str) -> Result<(), TypeError> {
        let cstr = CString::new(name).map_err(|_| TypeError::BadString)?;
        let exists = unsafe { duk_get_global_string(self.ptr(), cstr.as_ptr()) } != 0;
        if exists {
            Ok(())
        } else {
            Err(TypeError::NoneType)
        }
    }

    fn get_string(&mut self, idx: i32) -> Result<String, TypeError> {
        let cstr = unsafe { duk_get_string(self.ptr(), idx) };
        let cstr = unsafe { CStr::from_ptr(cstr) };
        let cstr = cstr.to_owned();
        cstr.into_string().map_err(|_| TypeError::BadString)
    }

    fn require_stack(&mut self, sz: i32) {
        unsafe {
            duk_require_stack(self.ptr(), sz);
        }
    }

    fn push_null(&mut self) {
        unsafe {
            duk_push_null(self.ptr());
        }
    }

    fn push_string(&mut self, name: &str) -> Result<(), TypeError> {
        let cstr = CString::new(name).map_err(|_| TypeError::BadString)?;
        unsafe {
            duk_push_string(self.ptr(), cstr.as_ptr());
        }
        Ok(())
    }

    fn push_bool(&mut self, value: bool) {
        let value = if value { 1 } else { 0 };
        unsafe {
            duk_push_boolean(self.ptr(), value);
        }
    }
    fn push_c_function(
        &mut self,
        name: &str,
        func: unsafe extern "C" fn(ctx: *mut duk_context) -> duk_ret_t,
        nargs: i32,
    ) -> Result<(), TypeError> {
        let name = CString::new(name).map_err(|_| TypeError::BadString)?;
        unsafe {
            duk_push_c_function(
                self.ptr(),
                /* func pointer */ Some(func),
                /* nargs: */ nargs,
            );
        }
        unsafe {
            duk_put_global_string(self.ptr(), name.as_ptr());
        }
        Ok(())
    }

    fn call(&mut self, nargs: i32) {
        unsafe {
            duk_call(self.ptr(), nargs);
        }
    }
}