use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};
use std::io::BufferedStream;

fn main () {

    let listener = TcpListener::bind("127.0.0.1", 8099).unwrap();
    let mut acceptor = listener.listen().unwrap();
    
    fn handle_client(mut stream: BufferedStream<TcpStream>) {
        stream.write(b"HTTP/1.1 200 OK\n"); // byte literal
        stream.write(b"Content-length: 15\n"); // byte literal
        stream.write(b"Content-type: text/html\n"); // byte literal
        stream.write(b"\n\n <p>Howdy!</p>");
        //stream.flush();

        println!("Handling acceptor");
    }

    for stream in acceptor.incoming() {
        match stream {
            Err(e) => { println!("connection failed: {}", e) }
            Ok(mut stream) => spawn(proc() {
                let mut bs = BufferedStream::new(stream);
                handle_client(bs)
            })
        }
    }

    drop(acceptor);
}
