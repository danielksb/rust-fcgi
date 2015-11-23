rust-fcgi
=========

This package provides a Rust binding to the C/C++ fast-cgi library.


# Basic Usage
 
Run `cargo build` to compile the example code under `examples/example.rs`.
This will provide you with a simple "Hello World" web service which is writing
some status information to stdout and an error message to the FCGI error stream.

To use the example programme simply configure your web server to run the binary
or connect to it via tcp, here is an example configuration for lighttpd:
 
```
fastcgi.server = (
      "/rust" => ((
             "host" => "127.0.0.1",
             "port" => 8080,
             "check-local" => "disable"
       ))
 )
```

Now you can start the FCGI process with
```
   spawn-fcgi target/fcgi-example -n -p 8080
```

Visit http://127.0.0.1/rust/hello to receive a welcoming greeting. You can also
try POSTing to the URL to test the `readall` method which should write the posted
request body to stdout:
```
     curl --request POST --data Test http://127.0.0.1/rust
```
