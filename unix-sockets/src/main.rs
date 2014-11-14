use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};
use std::io::BufferedStream;
use std::string::String;

fn main () {

    let listener = TcpListener::bind("127.0.0.1", 8099).unwrap();
    let mut acceptor = listener.listen().unwrap();
    
    fn handle_client(mut stream: BufferedStream<TcpStream>) {
        let s = String::from_str("X-header: from bytes\n");


        stream.write(b"HTTP/1.1 200 OK\n").unwrap(); // byte literal
        stream.write(b"Content-length: 21\n").unwrap(); // byte literal
        stream.write(b"Content-type: text/html\n").unwrap(); // byte literal
        stream.write(s.into_bytes().as_slice());
        stream.write(b"\n\n <p>Howdy There!</p>").unwrap();
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
