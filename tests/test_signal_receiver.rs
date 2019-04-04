use std::thread;
use sonr::prelude::*;
use sonr::sync::signal::{SignalReceiver, ReactiveSignalReceiver, SignalSender};
use sonr::sync::Capacity;


#[test]
fn test_signal_receiver() {
    let rx: SignalReceiver<u8> = SignalReceiver::unbounded();
    let tx = rx.sender();

    let handle = thread::spawn(move || {
        let handle = System::init().unwrap();
        let rx = ReactiveSignalReceiver::new(rx).unwrap();
        
        let run = rx.map(|val| {
            assert_eq!(val, 123);
            handle.send(SystemEvent::Stop);
        });
        System::start(run);
    });

    tx.send(123);

    handle.join();
}
