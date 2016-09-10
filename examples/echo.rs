extern crate fcgi;
extern crate libc;

use fcgi::{Request, DefaultRequest};
use std::sync::{Arc, Mutex};
use std::thread;


static NTASKS: i32 = 8;

fn handle_request(accept_lock: Arc<Mutex<i32>> ) {
    let mut request: DefaultRequest = Request::new().unwrap();
    loop {
        {
            let _ = accept_lock.lock();
            if !request.accept() {
                break;
            }
        }
        let received = request.readall();
        request.write("Content-type: text/plain\r\n");
        request.write("\r\n");
        request.write(received.as_ref());
        request.flush(fcgi::StreamType::OutStream);
        request.finish();
    }
}

fn main() {
	
    fcgi::initialize_fcgi();

    let accept_lock = Arc::new(Mutex::new(0));

    for _ in 0..NTASKS {
        let child_accept_lock = accept_lock.clone();
        let t = thread::spawn(move || handle_request(child_accept_lock));
        t.join().unwrap();
    }
}

