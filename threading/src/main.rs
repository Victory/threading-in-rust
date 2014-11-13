extern crate native;

use std::comm;
use std::io::timer;
use std::time::duration::Duration;

fn main() {
    let (tx, rx): (Sender<uint>, Receiver<uint>) = comm::channel();

    for ii in range(0u, 10) {
        let interval = Duration::milliseconds(ii as i64 * 20i64);
        let task_tx = tx.clone(); // clone the Sender

        spawn(proc() {
            task_tx.send(ii);
            timer::sleep(interval);
            println!("task {} is done", ii);
        });
    }

    for _ in range(0u, 10) {
        println!("Recieved {}", rx.recv());
    }
}
