use std::io;
use std::thread;

use mio::Event;

use sonr::reactor::{Reactive, Reaction};
use sonr::sync::broadcast::Broadcast;
use sonr::sync::signal::{SignalSender, ReactiveSignalReceiver};
use sonr::sync::queue::{ReactiveQueue, ReactiveDeque};
use sonr::system::{System, SystemEvent};
use sonr::reactor::producers::ReactiveGenerator;

#[derive(Debug)]
struct Counter {
    sender: SignalSender<SystemEvent>,
    counter: u8,
}

impl Reactive for Counter {
    type Output = ();
    type Input = String;

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> { 
        self.counter += 1;
        //eprintln!("------ count: {} - {:?}", self.counter, reaction);
        if self.counter == 4 {
            self.sender.send(SystemEvent::Stop);
        }
        Reaction::NoReaction 
    }
}

#[test]
fn test_broadcast() { 
    // -----------------------------------------------------------------------------
    // 		- Broadcast a message from two threads to two different threads -
    // 		Each receiving thread should receive two messages and then
    // 		signal the system to stop
    // -----------------------------------------------------------------------------
    let bc = Broadcast::<String>::unbounded();
    let bc1 = bc.clone();
    let bc2 = bc.clone();
    
    // First receiving thread
    let s1 = bc.subscriber();
    let h1 = thread::spawn(move || {
        let sender = System::init().unwrap();
        let counter = Counter { sender, counter: 0 };
        let subscriber = ReactiveSignalReceiver::new(s1).unwrap().map(|v| {
            eprintln!("-> thread 1: : {:?}", v);
            v
        }).chain(counter);
        System::start(subscriber);
    });

    // Second receiving thread
    let s2 = bc.subscriber();
    let h2 = thread::spawn(move || {
        let sender = System::init().unwrap();
        let counter = Counter { sender, counter: 0 };
        let subscriber = ReactiveSignalReceiver::new(s2).unwrap().map(|v| {
            eprintln!("-> thread 2: : {:?}", v);
            v
        }).chain(counter);
        System::start(subscriber);
    });

    // Give the queue some time
    thread::sleep_ms(10);

    // First broadcasting thread
    let h3 = thread::spawn(move || {
        bc1.publish("first broadcast".into());
        bc1.publish("ofloff broadcast".into());
        bc1.publish("magic broadcast".into());
        bc1.publish("pagic broadcast".into());
    });

    // Second broadcasting thread
    let h4 = thread::spawn(move || {
        bc2.publish("second broadcast".into());
        bc2.publish("second magic broadcast".into());
        bc2.publish("second ofloff broadcast".into());
        bc2.publish("second pagic broadcast".into());
    });

    h1.join();
    h2.join();
    h3.join();
    h4.join();
}

#[test]
fn test_bounded_queue() {
    let handle = System::init().unwrap();
    let gen = ReactiveGenerator::new((1u8..=4).collect()).unwrap();
    let mut queue = ReactiveQueue::bounded(1);

    let deque = queue.deque();

    let thread_handle = thread::spawn(move || {
        thread::sleep_ms(30);
        let fo_handle = System::init().unwrap();
        let dq = ReactiveDeque::new(deque).unwrap();
        let run = dq.map(|s| {
            eprintln!("<- rx: {:?}", s);
            if s == 4 {
                let fo_handle = fo_handle.clone();
                thread::spawn(move || {
                    fo_handle.send(SystemEvent::Stop);
                });
            }
        });
        System::start(run);
        eprintln!("{:?}", "DONE");
    });

    let run = gen.map(|i| {
        if i == 4 {
            handle.send(SystemEvent::Stop);
        }
        i
    }).chain(queue);
    System::start(run);
    thread_handle.join();
}


#[test]
fn test_bounded_broadcast() { 
    // -----------------------------------------------------------------------------
    // 		- Broadcast a message from two threads to two different threads -
    // 		Each receiving thread should receive two messages and then
    // 		signal the system to stop
    // -----------------------------------------------------------------------------
    let bc = Broadcast::<String>::bounded(0);
    let bc1 = bc.clone();
    let bc2 = bc.clone();
    
    // First receiving thread
    let s1 = bc.subscriber();
    let h1 = thread::spawn(move || {
        let sender = System::init().unwrap();
        let counter = Counter { sender, counter: 0 };
        let subscriber = ReactiveSignalReceiver::new(s1).unwrap().map(|v| {
            eprintln!("-> thread 1: : {:?}", v);
            v
        }).chain(counter);
        System::start(subscriber);
    });

    // Second receiving thread
    let s2 = bc.subscriber();
    let h2 = thread::spawn(move || {
        let sender = System::init().unwrap();
        let counter = Counter { sender, counter: 0 };
        let subscriber = ReactiveSignalReceiver::new(s2).unwrap().map(|v| {
            eprintln!("-> thread 2: : {:?}", v);
            v
        }).chain(counter);
        System::start(subscriber);
    });

    // First broadcasting thread
    let h3 = thread::spawn(move || {
        bc1.publish("first broadcast".into());
    });

    // Second broadcasting thread
    let h4 = thread::spawn(move || {
        bc2.publish("second broadcast".into());
    });

    h1.join();
    h2.join();
    h3.join();
    h4.join();
}
