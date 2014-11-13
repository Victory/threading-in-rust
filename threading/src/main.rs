extern crate native;

use std::comm;


fn main() {
    let (tx, rx): (Sender<uint>, Receiver<uint>) = comm::channel();



    for ii in range(0u, 10) {
        let task_tx = tx.clone(); // clone the Sender

        spawn(proc() {
            task_tx.send(ii);

            println!("task {} is done", ii);
        });
    }

    for _ in range(0u, 10) {
        println!("Recieved {}", rx.recv());
    }
}
