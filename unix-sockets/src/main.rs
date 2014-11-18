use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};
use std::io::BufferedStream;
use std::string::String;


fn parse_request_line(header: &str) {
    let mut pathname = String::new();
    let mut method = String::new();
    let mut starting = true;
    let mut found_method = false;

    for ch in header.graphemes(true) {
        if starting && ch != " " {
            method.push_str(ch);
        } 
        if starting && ch == " " {
            starting = false;
            found_method = true;
            continue;
        }
        if found_method && ch != " " {
            pathname.push_str(ch);
        }
        if found_method && ch == " " {
            break;
        }

    }

    println!("METHOD: {}", method);
    println!("PATHNAME: {}", pathname);
}

fn main () {

    let listener = TcpListener::bind("127.0.0.1", 8099).unwrap();
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
            if cur_line.len() == 2 { // TODO: make this check for \n\n explicitly
                break;
            }
        }

        let body_length = format!("Content-length: {}", body.len());

        stream.write(b"HTTP/1.1 200 OK\n").unwrap(); // byte literal
        stream.write(b"Content-type: text/html\n").unwrap();
        stream.write(b"X-header: from bytes\n").unwrap();
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
