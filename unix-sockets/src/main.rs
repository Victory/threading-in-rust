use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};
use std::io::BufferedStream;
use std::string::String;

fn main () {

    let listener = TcpListener::bind("127.0.0.1", 8099).unwrap();
    let mut acceptor = listener.listen().unwrap();
    
    fn handle_client(mut stream: BufferedStream<TcpStream>) {
        let body = String::from_str("<p>Hi, World!</p>");
        let body_length = format!("Content-length: {}", body.len());

        stream.write(b"HTTP/1.1 200 OK\n").unwrap(); // byte literal
        stream.write(b"Content-type: text/html\n").unwrap();
        stream.write(b"X-header: from bytes\n");
        stream.write(body_length.into_bytes().as_slice()).unwrap(); 
        stream.write(b"\n\n").unwrap();
        stream.write(body.into_bytes().as_slice()).unwrap();
        stream.flush().unwrap();

        println!("Handling acceptor");
    }

    for stream in acceptor.incoming() {
        match stream {
            Err(e) => { println!("connection failed: {}", e) }
            Ok(stream) => spawn(proc() {
                let bs = BufferedStream::new(stream);
                handle_client(bs)
            })
        }
    }

    drop(acceptor);
}
