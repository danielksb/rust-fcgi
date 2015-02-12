
#![crate_name = "fcgi"]
#![crate_type = "lib"]

//! This package provides a Rust binding to the C/C++ [fast-cgi library][]
//!
//! [fast-cgi library]: http://www.fastcgi.com/devkit/doc/overview.html
//!
//! The low level API is available via `mod capi`. A safer rust interface is
//! provided by `mod core`. More method bindings and higher level API functions
//! will be added in the future.
//!
//! # Basic Usage
//! 
//! Run `cargo build` to compile the example code under `examples/example.rs`.
//! This will provide you with a simple "Hello World" web service which is writing
//! some status information to stdout and an error message to the FCGI error stream.
//!
//! To use the example programme simply configure your web server to run the binary
//! or connect to it via tcp, here is an example configuration for lighttpd:
//! 
//! ```
//! fastcgi.server = (
//!        "/cpp" => ((
//!                "host" => "127.0.0.1",
//!                "port" => 8080,
//!                "max-procs" => "1",
//!                "check-local" => "disable"
//!         ))
//! )
//! ```
//!
//! Now you can start the FCGI process with
//! ```
//!     spawn-fcgi target/fcgi-example -n -p 8080
//! ```
//!
//! Visit http://127.0.0.1/cpp/hello to receive a welcoming greeting. You can also
//! try POSTing to the URL to test the `readall` method which should write the posted
//! request body to stdout:
//! ```
//!     curl --request POST --data Test http://127.0.0.1/cpp/hello
//! ```

extern crate libc;
use std::default::Default;
use std::ffi;
use std::str;


pub mod capi;

/// Initialize the FCGX library. Returns true upon success.
pub fn initialize_fcgi() -> bool {
    unsafe {
        return capi::FCGX_Init() == 0;
    }
}

/// Returns true if this process appears to be a CGI process 
/// rather than a FastCGI process.
pub fn is_cgi() -> bool {
    unsafe {
        return capi::FCGX_IsCGI() != 0;
    }
}

#[deriving(Copy)]
pub enum StreamType { OutStream, InStream, ErrStream }

/// Methods for working with an FCGI request object. A default implementation is provided within this package.
pub trait Request {

    /// Creates a new already initialized instance of an FCGI request.
    fn new() -> Option<Self>;

    /// Accept a new request (multi-thread safe).  Be sure to call initialize_fcgi() first.
    fn accept(&mut self) -> bool;
    
    /// Finish the request (multi-thread safe).
    fn finish(&mut self);

    /// Get a value of a FCGI parameter from the environment.
    fn get_param(&self, name: &str) -> Option<String>;

    /// Writes the given String into the output stream.
    fn write(&mut self, msg: &str) -> i32;

    /// Writes the given String into the error stream.
    fn error(&mut self, msg: &str) -> i32;

    /// Reads the entire input into a String, returns the
    /// empty string of no input was read.
    fn readall(&mut self) -> String;

    /// Reads up to n consecutive bytes from the input stream
    /// and returns them as String.  Performs no interpretation
    /// of the input bytes. The second value of the returned 
    /// tuple is the number of bytes read from the stream. If the
    /// result is smaller than n, the end of input has been reached.
    fn read(&mut self, n: i32) -> (String, i32);

    /// Flushes any buffered output
    fn flush(&mut self, stream_type: StreamType);
}

/// Default implementation for FCGI request
#[allow(missing_copy_implementations)]
pub struct DefaultRequest {
    raw_request: capi::FCGX_Request
}

impl Request for DefaultRequest {
    fn new() -> Option<DefaultRequest> {
        let mut request: capi::FCGX_Request = Default::default();
        unsafe {
            if capi::FCGX_InitRequest(&mut request, 0, 0) == 0 {
                return Some(DefaultRequest {raw_request: request });
            } else {
                return None;
            }
        }
    }

    fn accept(&mut self) -> bool {
        unsafe {
            return capi::FCGX_Accept_r(&mut self.raw_request) == 0;
        }
    }

    fn finish(&mut self) {
        unsafe {
            capi::FCGX_Finish_r(&mut self.raw_request);
        }
    }

    fn get_param(&self, name: &str) -> Option<String> {
        let cstr = ffi::CString::from_slice(name.as_bytes());
        unsafe {
            let param = capi::FCGX_GetParam(cstr.as_ptr(), self.raw_request.envp);
            if param.is_null() {
                return None;
            }
            let resultStr = str::from_c_str(param);
            return Some(String::from_str(resultStr));
        }
    }

    fn write(&mut self, msg: &str) -> i32 {
        let cstr = ffi::CString::from_slice(msg.as_bytes());
        unsafe {
            return capi::FCGX_PutS(cstr.as_ptr(), self.raw_request.out_stream);
        }
    }

    fn error(&mut self, msg: &str) -> i32 {
        let cstr = ffi::CString::from_slice(msg.as_bytes());
        unsafe {
            return capi::FCGX_PutS(cstr.as_ptr(), self.raw_request.err_stream);
        }
    }

    fn read(&mut self, n: i32) -> (String, i32) {
        unsafe {
            let size = (n + 1) as usize;
            let mut buffer = Vec::with_capacity(size);
            let pdst = buffer.as_mut_ptr();
            let byte_count = capi::FCGX_GetStr(pdst, n, self.raw_request.in_stream);
            buffer.set_len(byte_count as usize);
            let resultStr = str::from_c_str(pdst);
            return (String::from_str(resultStr), byte_count);
        }
    }
    
    fn readall(&mut self) -> String {
        let (mut msg, mut n) = self.read(512);
        while n == 512 {
            let (new_msg, new_n) = self.read(512);
            msg = msg + new_msg.as_slice();
            n = new_n;
        }
        return msg;
    }

    fn flush(&mut self, stream_type: StreamType) {
        let stream = match stream_type {
            StreamType::OutStream => self.raw_request.out_stream,
            StreamType::InStream  => self.raw_request.in_stream,
            StreamType::ErrStream => self.raw_request.err_stream,
        };
        unsafe {
            capi::FCGX_FFlush(stream);
        }
    }
}

