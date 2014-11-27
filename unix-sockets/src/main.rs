extern crate "rust-crypto" as rust_crypto;
extern crate serialize;

use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};
use std::io::{File, BufferedStream};
use std::string::String;

use rust_crypto::sha1::Sha1;
use rust_crypto::digest::Digest;
use serialize::base64::{ToBase64, STANDARD};


pub const CR: u8 = b'\r';
pub const LF: u8 = b'\n';
pub const SP: u8 = b' ';
pub const CRLF: &'static [u8] = &[CR,LF];

struct RequestedRoute {
    method: String,
    pathname: String
}

fn parse_request_line(header: &str) -> RequestedRoute {
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
    return RequestedRoute {method: method, pathname: pathname};
}

fn get_normal_body (path_on_disk: &str) -> String {
    let path = Path::new(path_on_disk);
    let display = path.display();
    let mut file = match File::open(&path) {
        Ok(f) => f,
        Err(err) => panic!("file error: {}", err)
    };

    let content = match file.read_to_end() {
        Ok(c) => c,
        Err(err) => panic!("{}", err)
    };

    let s = String::from_utf8(content).unwrap();
    return s;
}

fn get_js_body () -> String {
    let path = Path::new("../html/ws1.js");
    let display = path.display();
    let mut file = match File::open(&path) {
        Ok(f) => f,
        Err(err) => panic!("file error: {}", err)
    };

    let content = match file.read_to_end() {
        Ok(c) => c,
        Err(err) => panic!("{}", err)
    };

    let s = String::from_utf8(content).unwrap();
    return s;
}


fn sec_handshake (from_server: &[u8]) -> String {

    let guid = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

    let mut sha = Sha1::new();

    sha.input(from_server);
    sha.input(guid);
    let mut out = [0u8, ..20];
    sha.result(out.as_mut_slice());

    println!("{} {}", sha.result_str(), out.to_base64(STANDARD));

    return out.to_base64(STANDARD);
}


fn main () {

    let listener = TcpListener::bind("127.0.0.1:8099").unwrap();
    let mut acceptor = listener.listen().unwrap();
    
    fn handle_client(mut stream: BufferedStream<TcpStream>) {

        let mut body: String = "<p>You sent it!</p>".to_string();

        let mut cur_line: String;
        let mut ii = 0u;
        let mut req: RequestedRoute = RequestedRoute{
            pathname: String::new(), 
            method: String::new()
        };

        loop {
            match stream.read_line() { // XXX: strange unwrap like thing
                Ok(line) => cur_line = line,
                Err(_) => break
            }
            if ii == 0u { // the Request-Line is always the first line
                req = parse_request_line(cur_line.as_slice());
                ii += 1;
            }

            body = body + cur_line + "<br>";
            if cur_line.as_bytes() == CRLF {
                break;
            }
        }

        println!("pathname {} method {}", req.pathname, req.method);
        let mut secHandshake = false;

        if req.pathname.as_bytes() == b"/" {
            body = get_normal_body("../html/ws1.html");
        } else if req.pathname.as_bytes() == b"/ws1.js" {
            body = get_js_body();
        } else if req.pathname.as_bytes() == b"/ws" {
            secHandshake = true;
            // TODO get handshake
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
