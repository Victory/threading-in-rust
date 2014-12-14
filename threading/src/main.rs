//extern crate native;

use std::comm;
use std::io::Timer;
use std::io::timer;
use std::time::duration::Duration;


fn subtx(ii: uint, interval: Duration, task_tx: Sender<uint>) {
    task_tx.send(ii);
    timer::sleep(interval);
    println!("task {} is done", ii);
}


fn main() {
    let (tx, rx): (Sender<uint>, Receiver<uint>) = comm::channel();
    let mut timer = Timer::new().unwrap();
    let interval = Duration::milliseconds(500);
    let tictoc: Receiver<()> = timer.periodic(interval);

    for ii in range(0u, 10) {
        let interval = Duration::milliseconds(ii as i64 * 20i64);
        let task_tx = tx.clone(); // clone the Sender
        spawn(proc() {
            subtx(ii, interval, task_tx.clone());
            subtx(ii, interval, task_tx.clone());
        });
    }

    for ii in std::iter::range_step(10i, 0, -1) {
        tictoc.recv();
        println!("{}", ii);
        println!("Recieved {}", rx.recv());
    }

    println!("go!");
}
