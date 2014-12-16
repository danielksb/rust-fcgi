
#![crate_name = "fcgi"]
#![crate_type = "lib"]

//! This package provides a Rust binding to the C/C++ [fast-cgi library][]
//!
//! [fast-cgi library]: http://www.fastcgi.com/devkit/doc/overview.html
//!
//! The low level API is available via `mod ffi`. A safer rust interface is
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


pub mod ffi;

/// Initialize the FCGX library. Returns true upon success.
pub fn initialize_fcgi() -> bool {
    unsafe {
        return ffi::FCGX_Init() == 0;
    }
}

/// Returns true if this process appears to be a CGI process 
/// rather than a FastCGI process.
pub fn is_cgi() -> bool {
    unsafe {
        return ffi::FCGX_IsCGI() != 0;
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
    raw_request: ffi::FCGX_Request
}

impl Request for DefaultRequest {
    fn new() -> Option<DefaultRequest> {
        let mut request: ffi::FCGX_Request = Default::default();
        unsafe {
            if ffi::FCGX_InitRequest(&mut request, 0, 0) == 0 {
                return Some(DefaultRequest {raw_request: request });
            } else {
                return None;
            }
        }
    }

    fn accept(&mut self) -> bool {
        unsafe {
            return ffi::FCGX_Accept_r(&mut self.raw_request) == 0;
        }
    }

    fn finish(&mut self) {
        unsafe {
            ffi::FCGX_Finish_r(&mut self.raw_request);
        }
    }

    fn get_param(&self, name: &str) -> Option<String> {
        unsafe {
            let param = ffi::FCGX_GetParam(name.to_c_str().as_ptr(), self.raw_request.envp);
            if param.is_null() {
                return None;
            }
            let paramstr = std::c_str::CString::new(param as *const libc::c_char, false);
            return Some(String::from_str(paramstr.as_str().unwrap_or("")));
        }
    }

    fn write(&mut self, msg: &str) -> i32 {
        unsafe {
            return ffi::FCGX_FPrintF(self.raw_request.out_stream, msg.to_c_str().as_ptr());
        }
    }

    fn error(&mut self, msg: &str) -> i32 {
        unsafe {
            return ffi::FCGX_FPrintF(self.raw_request.err_stream, msg.to_c_str().as_ptr());
        }
    }

    fn read(&mut self, n: i32) -> (String, i32) {
        unsafe {
            let input = libc::malloc(n as libc::size_t) as *mut libc::c_char;
            let byte_count = ffi::FCGX_GetStr(input, n, self.raw_request.in_stream);
            let output = std::c_str::CString::new(input as *const libc::c_char, false);
            return (String::from_str(output.as_str().unwrap_or("")), byte_count);
        }
    }
    
    fn readall(&mut self) -> String {
        let (mut msg, mut n) = self.read(512);
        while n == 512 {
            let (new_msg, new_n) = self.read(512);
            msg = msg + new_msg;
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
            ffi::FCGX_FFlush(stream);
        }
    }
}

