extern crate "rust-crypto" as rust_crypto;
extern crate serialize;

use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};
use std::io::{File, BufferedStream};
use std::string::String;
use std::os;
use std::vec::Vec;

use rust_crypto::sha1::Sha1;
use rust_crypto::digest::Digest;
use serialize::base64::{ToBase64, STANDARD};

use std::io::Timer;
use std::io::timer;
use std::time::duration::Duration;

pub const CR: u8 = b'\r';
pub const LF: u8 = b'\n';
pub const SP: u8 = b' ';
pub const COLON: u8 = b':';
pub const CRLF: &'static [u8] = &[CR,LF];


struct RequestedRoute {
    method: String,
    pathname: String
}


struct ClientHeader {
    key: String,
    value: String
}


enum Payload {
    Text(String),
    Binary(Vec<u8>),
    Empty
}


enum Opcode {
    EmptyOp = 0x0,
    TextOp = 0x1,
    BinaryOp = 0x2,
    CloseOp = 0x8,
}


struct Message {
    payload: Payload,
    opcode: Opcode,
    fin: u8
 }


impl Message {

    fn from_payload (payload: Payload, fin: u8) -> Message {
        let opcode: Opcode = match payload {
            Payload::Text(_) => Opcode::TextOp,
            Payload::Binary(_) => Opcode::BinaryOp,
            Payload::Empty => Opcode::EmptyOp
        };
        return Message {payload: payload, opcode: opcode, fin: fin};
    }

    fn continue_from_payload (payload: Payload) -> Message {
        let opcode = Opcode::EmptyOp;
        let fin = 0x0;
        return Message {payload: payload, opcode: opcode, fin: fin};
    }

    fn send (&self,
            mut stream: &mut BufferedStream<TcpStream>) {
        let msg = match self.payload {
            Payload::Text(ref s) => s.as_bytes(),
            Payload::Binary(ref s) => s.as_slice(),
            Payload::Empty => "".as_bytes(),
        };

        let length = msg.len() as u8;

        println!("fin: {}, msg: {}, opcode: {}", self.fin, msg, self.opcode as u8);

        stream.write_u8(self.fin | self.opcode as u8).unwrap();
        stream.write_u8(length).unwrap();
        stream.write(msg).unwrap();
        stream.flush();
    }

    fn from_stream(mut stream: &mut BufferedStream<TcpStream>) -> Message {
        let cur_byte: u8 = stream.read_byte().unwrap();

        let fin = cur_byte & 0b1000_0000;
        let rsv = cur_byte & 0b0111_0000;
        let opc = cur_byte & 0b0000_1111;
        let msk = cur_byte & 0b0000_0001;

        let cur_byte: u8 = stream.read_byte().unwrap();
        let len = (cur_byte & 0b0111_1111) as uint;

        let mskkey = stream.read_exact(4).unwrap();
        
        let mut msg = Vec::new();
        for ii in range(0u, len) {
            let cur_byte: u8 = stream.read_byte().unwrap();
            let ch = mskkey[ii % 4] ^ cur_byte;
            msg.push(ch);
        }


        let utf8_msg = match String::from_utf8(msg) {
            Ok(m) => m,
            Err(_) => panic!("I don't know how to ut8 that")
        };

        println!(
            "fin {}, rsv {}, msk {}, opcode {}, len {}, mskkey {}, msg {}", 
            fin, rsv, msk, opc, len, mskkey, utf8_msg);
        
        let payload = Payload::Text(utf8_msg);
        return Message::from_payload(payload, fin);
    }

    fn from_buffer (buf: &[u8]) {
        let fin = buf[0] & 0b1000_0000;
        let rsv = buf[0] & 0b0111_0000;
        let opc = buf[0] & 0b0000_1111;
        let msk = buf[0] & 0b0000_0001;
        let len = (buf[1] & 0b0111_1111) as uint;
        let mskkey = buf.slice(2, 6);

        let mut msg = Vec::new();
        for ii in range(0u, len) {
            let ch = mskkey[ii % 4] ^ buf[6 + ii];
            msg.push(ch);
        }

        println!(
            "fin {}, rsv {}, msk {}, opcode {}, len {}, mskkey {}, msg {}, \nbuf {}", 
            fin, rsv, msk, opc, len, mskkey, String::from_utf8(msg),  buf.as_slice());
    }
}

fn get_header_by_name (header: &[u8], headers: &Vec<ClientHeader>) -> String {
    let mut result = String::new();

    for h in headers.iter() {
        if h.key.as_bytes() == header {
            result = h.value.clone();
            break;
        }
    }

    return result;
}


fn parse_request_line (header: &str) -> RequestedRoute {
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


fn parse_normal_header (header: &str) -> ClientHeader {
    let mut lhs = String::new(); // lhs of ':' in header
    let mut rhs = String::new(); // rhs of ':' in header
    let mut found_colon = false;

    for ch in header.graphemes(true) {
        let cbyte = ch.as_bytes()[0];
        if !found_colon && cbyte == COLON {
            found_colon = true;
            continue;
        }

        if cbyte == CR || cbyte == LF {
            continue;
        }

        if !found_colon {
            lhs.push_str(ch);
        } else {
            rhs.push_str(ch);
        }
    }


    return ClientHeader {
        key:lhs.trim().to_string(),
        value:rhs.trim().to_string()
    };
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


fn sec_handshake (from_server: &[u8]) -> String {

    // from rfc6455 [page 6]
    let guid = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

    let mut sha = Sha1::new();

    sha.input(from_server);
    sha.input(guid);
    let mut out = [0u8, ..20];
    sha.result(out.as_mut_slice());

    return out.to_base64(STANDARD);
}


fn ws_handshake (mut stream: BufferedStream<TcpStream>,
                 headers: Vec<ClientHeader>) {

    let header_sec_key = b"Sec-WebSocket-Key";

    let from_server = get_header_by_name(header_sec_key, &headers);
    let accept = sec_handshake(from_server.as_bytes());

    let sec_header = format!("Sec-WebSocket-Accept: {}\r\n", accept);

    stream.write(b"HTTP/1.1 101 Switching Protocols\r\n").unwrap();
    stream.write(b"Upgrade: websocket\r\n").unwrap();
    stream.write(b"Connection: Upgrade\r\n").unwrap();
    stream.write(b"Sec-WebSocket-Version: 13\r\n").unwrap();
    stream.write(b"Sec-WebSocket-Protocol: protocolOne\r\n").unwrap();
    stream.write(sec_header.as_bytes()).unwrap();
    stream.write(b"\r\n");

    ws_listen(stream, headers);
}


fn ws_listen(mut stream: BufferedStream<TcpStream>,
             mut headers: Vec<ClientHeader>) {


    let payload = Payload::Text("text ".to_string());
    let msg = Message::from_payload(payload, 0b0000_0000);
    msg.send(&mut stream);

    let payload = Payload::Text("here".to_string());
    let msg = Message::continue_from_payload(payload);
    let mut stream2 = stream;
    msg.send(&mut stream2);

    let payload = Payload::Empty;
    let msg = Message::from_payload(payload, 0b1000_0000);
    let mut stream3 = stream2;
    msg.send(&mut stream3);

    println!("done sending");

    let mut stream4 = stream3;
    Message::from_stream(&mut stream4);

    let mut stream5 = stream4;
    let echo_msg = Message::from_stream(&mut stream5);

    echo_msg.send(&mut stream4);

    echo_msg.send(&mut stream5);




    /*
    let mut timer = Timer::new().unwrap();
    let interval = Duration::milliseconds(5000);
    timer::sleep(interval);
    */

    /*
    let mut stream4 = stream3;
    let mut buf = [0, ..100];
    match stream4.read(&mut buf) {
        Ok(nread) => println!("Read {} bytes", nread),
        Err(e) => println!("error reading: {}", e)
    }

    Message::from_buffer(&buf);

     */

    
    
}


fn main () {

    for argument in os::args().iter() {
        println!("arg: {}", argument);
    }

    let addr = "127.0.0.1:8099";

    let listener = TcpListener::bind(addr).unwrap();
    let mut acceptor = listener.listen().unwrap();
    println!("Listening on {}", addr);

    fn handle_client(mut stream: BufferedStream<TcpStream>) {
        let mut body: String = "<p>You sent it!</p>".to_string();

        let mut cur_line: String;
        let mut ii = 0u;
        let mut req: RequestedRoute = RequestedRoute {
            pathname: String::new(),
            method: String::new()
        };


        let mut headers: Vec<ClientHeader> = Vec::new();
        loop {
            match stream.read_line() { // XXX: strange unwrap like thing
                Ok(line) => cur_line = line,
                Err(_) => break
            }

            body = body + cur_line + "<br>";
            if cur_line.as_bytes() == CRLF {
                break;
            }

            if ii == 0u { // the Request-Line is always the first line
                req = parse_request_line(cur_line.as_slice());
                println!("{}", req.pathname);
                ii += 1;
            } else { // this should be  "normal" key-value type header
                headers.push(parse_normal_header(cur_line.as_slice()));
            }

        }

        println!("pathname {} method {}", req.pathname, req.method);

        if req.pathname.as_bytes() == b"/" {
            body = get_normal_body("./html/ws1.html");
        } else if req.pathname.as_bytes() == b"/ws1.js" {
            body = get_normal_body("./html/ws1.js");
        } else if req.pathname.as_bytes() == b"/ws" {
            ws_handshake(stream, headers);
            return;
        }

        let body_length = format!("Content-length: {}", body.len());

        stream.write(b"HTTP/1.1 200 OK\r\n").unwrap();
        stream.write(b"Content-type: text/html\r\n").unwrap();
        stream.write(b"X-header: from bytes\r\n").unwrap();
        stream.write(body_length.as_bytes()).unwrap();
        stream.write(b"\r\n\r\n").unwrap();
        stream.write(body.as_bytes()).unwrap();
        stream.flush().unwrap();

        println!("Done handling acceptor");
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
