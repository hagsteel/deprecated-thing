//! Reactive queue / dequeue
use std::io;

use crossbeam::deque::{Worker, Stealer, Steal}; 
use mio::{Poll, Evented, Ready, PollOpt, Token};

use crate::sync::signal::{SignalReceiver, SignalSender}; 
use crate::reactor::{Reactor, EventedReactor, Reaction};
use crate::errors::Result;

use super::Capacity;

// -----------------------------------------------------------------------------
// 		- Reactive queue -
// -----------------------------------------------------------------------------
/// A reactive work stealing queue.
///
/// Since the queue can be unbounded or bounded it's possible to use a bounded queue 
/// to create back pressure.
///
/// An unbounded queue will push data as soon as it reacts with a `Reaction::Value`,
/// however with a bounded queue the queue can not continue to react until the value has been
/// consumed by the receiving end.
///
/// ```
/// # use std::thread;
/// # use std::time::Duration;
/// # use std::net::TcpStream as StdStream;
/// # use sonr::prelude::*;
/// # use sonr::errors::Result;
/// use sonr::sync::signal::SignalSender;
/// use sonr::sync::queue::{ReactiveQueue, ReactiveDeque};
/// use sonr::net::tcp::{ReactiveTcpListener, TcpStream};
///
/// fn main() -> Result<()> {
///     let system_tx = System::init()?;
/// 
///     let listener = ReactiveTcpListener::bind("127.0.0.1:5556")?
///         .map(|(s, _)| {
///             // Put a system tx and the stream in the queue
///             (system_tx.clone(), s)
///         }); 
///
///     let mut queue = ReactiveQueue::bounded(1);
///     // Note the deque created here is not a ReactiveDeque but rather
///     // a Deque, as a ReactiveDeque can not be created in one thread
///     // and sent to another
///     let deque = queue.deque();
///
///     # thread::spawn(move || {
///     #     thread::sleep(Duration::from_millis(100));
///     #     StdStream::connect("127.0.0.1:5556");
///     # });  
///     thread::spawn(move || -> Result<()> {
///         let system_tx = System::init()?;
///         let run = ReactiveDeque::new(deque)?
///             .map(|(tx, stream): (SignalSender<SystemEvent>, _)| {
///                 tx.send(SystemEvent::Stop);
///                 system_tx.send(SystemEvent::Stop);
///             });
///         System::start(run)?;
///         Ok(())
///     });
///
///     let run = listener.chain(queue);
/// 
///     System::start(run)?;
///     Ok(())
/// }
/// ```
pub struct ReactiveQueue<T> {
    inner: Queue<T>,
}

impl<T: Send + 'static> ReactiveQueue<T> { 
    /// Create an unbounded reactive queue
    pub fn unbounded() -> Self {
        Self {
            inner: Queue::unbounded(),
        }
    }

    /// Create an bounded reactive queue.
    /// Setting the capacity to zero means no data will be held in the 
    /// queue and the current thread will block until the data is picked up
    /// at the other end.
    pub fn bounded(capacity: usize) -> Self {
        Self {
            inner: Queue::bounded(capacity),
        }
    }

    /// Push a value onto the queue
    pub fn push(&self, val: T) {
        self.inner.push(val);
    }

    /// Create an instance of a [`Dequeue`].
    /// A [`Dequeue`] is not as useful in it self but
    /// rather the underlying deque of a [`ReactiveDeque`].
    ///
    /// [`Dequeu`]: struct.Deque.html
    /// [`ReactiveDeque`]: struct.ReactiveDeque.html
    pub fn deque(&mut self) -> Dequeue<T> {
        self.inner.deque()
    }
}

impl<T: Send + 'static> Reactor for ReactiveQueue<T> {
    type Output = ();
    type Input = T;

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        if let Reaction::Value(value) = reaction {
            self.push(value);
            Reaction::Value(())
        } else {
            Reaction::Continue
        }
    }
}


// -----------------------------------------------------------------------------
// 		- Work-stealing Queue -
// -----------------------------------------------------------------------------
/// An evented work stealing queue.
pub struct Queue<T> {
    worker: Worker<T>,
    inner_stealer: Stealer<T>,
    publishers: Vec<SignalSender<()>>,
    capacity: Capacity,
}

impl<T: Send + 'static> Queue<T> {
    fn new_with_capacity(capacity: Capacity) -> Self {
        let worker = Worker::new_lifo();
        let inner_stealer = worker.stealer();

        Self { 
            worker,
            inner_stealer,
            publishers: Vec::new(),
            capacity,
        }
    }

    /// Create an unbounded queue
    pub fn unbounded() -> Self {
        Self::new_with_capacity(Capacity::Unbounded)
    }

    /// Create a bounded queue.
    /// Setting the capacity to zero means no data will be held in the 
    /// queue and the current thread will block until the data is picked up
    /// at the other end.
    pub fn bounded(cap: usize) -> Self {
        Self::new_with_capacity(Capacity::Bounded(cap))
    }

    /// Push a value onto the queue
    pub fn push(&self, val: T) {
        self.worker.push(val);
        // Notify all
        self.publishers.iter().for_each(|p| { 
            let _ = p.send(()); 
        });
    }

    /// Create an instance of a [`Dequeu`].
    /// 
    /// [`Dequeu`]: struct.Deque.html
    pub fn deque(&mut self) -> Dequeue<T> {
        let stealer = self.inner_stealer.clone();
        let subscriber = match self.capacity {
            Capacity::Unbounded => Dequeue::unbounded(stealer),
            Capacity::Bounded(cap) => Dequeue::bounded(stealer, cap)
        };
        self.publishers.push(subscriber.sender());
        subscriber
    }
}

// -----------------------------------------------------------------------------
// 		- Reactive dequeue -
// ----------------------------------------------------------------------------- 
/// A reactive dequeue.
pub struct ReactiveDeque<T> {
    inner: EventedReactor<Dequeue<T>>,
}

impl<T> ReactiveDeque<T> { 
    /// Create a new reactive dequeue fro man existing [`Dequeue`]
    ///
    /// [`Dequeue`]: struct.Dequeue.html
    pub fn new(deq: Dequeue<T>) -> Result<Self> {
        Ok(Self {
            inner: EventedReactor::new(deq, Ready::readable())?,
        })
    }

    fn steal(&self) -> Reaction<T> {
        loop {
            match self.inner.inner().steal() {
                Steal::Retry => continue,
                Steal::Success(val) => break Reaction::Value(val),
                Steal::Empty => break Reaction::Continue,
            }
        }
    }
}

impl<T> Reactor for ReactiveDeque<T> {
    type Output = T;
    type Input = ();

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        match reaction {
            Reaction::Event(event) => {
                if event.token() != self.inner.token() {
                    return Reaction::Event(event);
                }
                self.steal()
            }
            Reaction::Value(_) => Reaction::Continue,
            Reaction::Continue => self.steal(),
        }
    }
}

// -----------------------------------------------------------------------------
// 		- Dequeue -
// -----------------------------------------------------------------------------
/// An evented work stealing dequeue
pub struct Dequeue<T> {
    signal: SignalReceiver<()>,
    stealer: Stealer<T>,
}

impl<T> Dequeue<T> {
    /// Create a bounded dequeue.
    /// Setting the capacity to zero means no data will be held in the 
    /// queue and the publishing thread will blocked until the data is picked up
    /// by the dequeue. 
    pub fn bounded(stealer: Stealer<T>, capacity: usize) -> Self {
        let signal = SignalReceiver::bounded(capacity);

        Self { 
            signal, 
            stealer,
        }
    }

    /// Create an unbounded dequeue 
    pub fn unbounded(stealer: Stealer<T>) -> Self {
        let signal = SignalReceiver::unbounded();

        Self { 
            signal, 
            stealer,
        }
    }

    /// Get the signal sender that notifies the dequeue of
    /// possible new data in the queue.
    pub fn sender(&self) -> SignalSender<()> {
        self.signal.sender()
    }

    /// Attempt to steal data
    pub fn steal(&self) -> Steal<T> {
        match self.signal.try_recv() {
            Ok(()) => {},
            Err(_e) => { /* dbg!(e); */ }
        }
        self.stealer.steal()
    }
}

impl<T> Evented for Dequeue<T> {
    fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        poll.register(
            &self.signal,
            token,
            interest,
            opts
        )
    }

    fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
        poll.reregister(
            &self.signal,
            token,
            interest,
            opts
        )
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        poll.deregister(&self.signal)
    }
}
