use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};
use std::io::BufferedStream;
use std::string::String;

pub const CR: u8 = b'\r';
pub const LF: u8 = b'\n';
pub const SP: u8 = b' ';
pub const CRLF: [u8, ..2] = [CR,LF];

fn parse_request_line(header: &str) {
    let mut pathname = String::new();
    let mut method = String::new();
    let mut starting = true;
    let mut found_method = false;

    for ch in header.graphemes(true) {
        let cbyte = ch.as_bytes()[0];

        if starting && cbyte != SP {
            method.push_str(ch);
        } 
        if starting && cbyte == SP {
            starting = false;
            found_method = true;
            continue;
        }
        if found_method && cbyte != SP {
            pathname.push_str(ch);
        }
        if found_method && cbyte == SP {
            break;
        }

    }

    println!("METHOD: {}", method);
    println!("PATHNAME: {}", pathname);
}

fn main () {

    let listener = TcpListener::bind("127.0.0.1:8099").unwrap();
    let mut acceptor = listener.listen().unwrap();
    
    fn handle_client(mut stream: BufferedStream<TcpStream>) {

        let mut body: String = "<p>You sent it!</p>".to_string();

        let mut cur_line: String;
        let mut ii = 0u;

        loop {
            match stream.read_line() { // XXX: strange unwrap like thing
                Ok(line) => cur_line = line,
                Err(_) => break
            }
            if ii == 0u { // the Request-Line is always the first line
                parse_request_line(cur_line.as_slice());
                ii += 1;
            }

            body = body + cur_line + "<br>";
            if cur_line.as_bytes() == CRLF {
                break;
            }
        }

        let body_length = format!("Content-length: {}", body.len());

        stream.write(b"HTTP/1.1 200 OK\r\n").unwrap(); // byte literal
        stream.write(b"Content-type: text/html\r\n").unwrap();
        stream.write(b"X-header: from bytes\r\n").unwrap();
        stream.write(body_length.into_bytes().as_slice()).unwrap(); 
        stream.write(b"\r\n\r\n").unwrap();
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
