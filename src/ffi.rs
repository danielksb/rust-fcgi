extern crate libc;

use std::default::Default;
use std::ptr::{null_mut};

#[allow(missing_copy_implementations)]
#[repr(C)]
pub struct FCGX_Request {
    pub request_id: libc::c_int,            /* valid if isBeginProcessed */
    pub role: libc::c_int,
    pub in_stream: *mut libc::c_void, // FCGX_Stream
    pub out_stream: *mut libc::c_void, // FCGX_Stream
    pub err_stream: *mut libc::c_void, // FCGX_Stream
	pub envp: *mut libc::c_void,

	/* Don't use anything below here */

    params_ptr: *mut libc::c_void,
    ipc_fd: libc::c_int,               /* < 0 means no connection */
    is_begin_processed: libc::c_int,     /* FCGI_BEGIN_REQUEST seen */
    keep_connection: libc::c_int,       /* don't close ipcFd at end of request */
    app_status: libc::c_int,
    writers: libc::c_int,             /* number of open writers (0..2) */
	flags: libc::c_int,
	listen_sock: libc::c_int,
}

impl Default for FCGX_Request {
    fn default() -> FCGX_Request {
        return FCGX_Request {
            request_id: 0,
            role: 0,
            in_stream: null_mut(),
            out_stream: null_mut(),
            err_stream: null_mut(),
            envp: null_mut(),
            params_ptr: null_mut(),
            ipc_fd: 0,
            is_begin_processed: 0,
            keep_connection: 0,
            app_status: 0,
            writers: 0,
            flags: 0,
            listen_sock: 0
        };
    }
}

#[link(name = "fcgi")]
extern {
    pub fn FCGX_IsCGI() -> libc::c_int;
    pub fn FCGX_Init() -> libc::c_int;
    pub fn FCGX_InitRequest(request: *mut FCGX_Request, sock: libc::c_int, flags: libc::c_int) -> libc::c_int;
    pub fn FCGX_Accept_r(request: *mut FCGX_Request) -> libc::c_int;
    pub fn FCGX_Finish_r(request: *mut FCGX_Request) -> libc::c_int;
    pub fn FCGX_GetParam(name: *const libc::c_char, envp: *mut libc::c_void) -> *mut libc::c_char;
    pub fn FCGX_FPrintF(stream: *mut libc::c_void, format: *const libc::c_char) -> libc::c_int;
    pub fn FCGX_GetStr(input: *mut libc::c_char, n: libc::c_int, stream: *mut libc::c_void) -> libc::c_int;
    pub fn FCGX_FFlush(stream: *mut libc::c_void);
}

