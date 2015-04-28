use std::cell::*;
use std::sync::*;
use std::sync::mpsc::*;
use std::marker::*;

use super::{Signal, Run};
use primitives::input::{RunInput, ReceiverInput};
use primitives::fork::{Fork, Branch};
use primitives::channel::Channel;
use primitives::async::Async;
use primitives::value::Value;

/// `Builder` is used to construct topologies.  
///
/// Basic builder pattern - `Topology::build` accepts a function which takes
/// a state type `T` and a mutable builder.  The builder can be used to create
/// `Channel`s and to `add` nodes to the topology
///
pub struct Builder {
    pub inputs: RefCell<Vec<Box<RunInput>>>,
    pub runners: RefCell<Vec<Box<Run>>>,
}

impl Builder {
    /// Create a new Builder
    ///
    pub fn new() -> Self {
        Builder {
            runners: RefCell::new(Vec::new()),
            inputs: RefCell::new(Vec::new()),
        }
    }

    /// Add a signal to the topology
    ///
    /// Returns a `Branch<A>`, allowing `root` to be used as input more than once
    ///
    /// # Example
    ///
    /// ```
    /// use cfrp::*;
    /// use cfrp::primitives::*;
    ///
    /// let b = Builder::new();
    /// 
    /// // Topologies only execute transformations which have been added to a builder.
    /// let fork = b.add(b.value(1).lift(|i| { i + 1} ));
    ///
    /// // `add` returns a signal that can be used more than once
    /// b.add(fork.clone().lift(|i| { i - 1 } ));
    /// b.add(fork.lift(|i| { -i }));
    /// ```
    ///
    pub fn add<SA, A>(&self, root: SA) -> Branch<A> where // NOTE: This needs to be clone-able!
        SA: 'static + Signal<A>,
        A: 'static + Clone + Send,
    {
        let v = root.initial();

        let fork_txs = Arc::new(Mutex::new(Vec::new()));

        let fork = Fork::new(Box::new(root), fork_txs.clone());

        self.runners.borrow_mut().push(Box::new(fork));

        Branch::new(fork_txs, None, v)
    }

    /// Listen to `input` and push received data into the topology
    ///
    /// All data must enter the topology via a call to `listen`; this function
    /// ensures data syncronization across the topology.  Each listener runs in 
    /// its own thread
    ///
    /// # Example
    ///
    /// ```
    /// use std::sync::mpsc::*;
    /// use cfrp::*;
    /// use cfrp::primitives::*;
    ///
    /// let b = Builder::new();
    /// 
    /// let (tx, rx): (Sender<usize>, Receiver<usize>) = channel();
    ///
    /// // Receive data on `rx` and expose it as a signal with initial value 
    /// //`initial`.  This is necessary because the topology must maintain 
    /// // consistency between threads, so any message sent to any input is 
    /// // propagated to all other inputs as "no-change" messages.
    /// let signal = b.listen(0, rx);
    /// ```
    ///
    pub fn listen<A>(&self, initial: A, input: Receiver<A>) -> Branch<A> where
        A: 'static + Clone + Send,
    {
        let (tx, rx) = channel();

        let runner = ReceiverInput::new(input, tx);

        self.inputs.borrow_mut().push(Box::new(runner));

        self.add(Channel::new(rx, initial))
    }

    /// Creats a channel with constant value `v`
    ///
    pub fn value<T>(&self, v: T) -> Value<T> where
        T: 'static + Clone + Send,
    {
        Value::new(v)
    }

    /// Combination of adding a signal and a channel
    ///
    /// Async allows signals to be processed downstream out of order.  Internally,
    /// the output of `root` is sent to new input channel.  The result is that
    /// long-running processes can be handled outside of the synchronized topology
    /// process, and the result can be handled when it's available.
    ///
    /// ```
    /// use std::sync::mpsc::*;
    /// use cfrp::*;
    /// use cfrp::primitives::*;
    ///
    /// let b = Builder::new();
    /// 
    /// // This will now happen without blocking the rest of the topology
    /// let result = b.async(
    ///     b.value(0).fold(0, |i, j| {
    ///         // Some very expensive code in here...
    ///     })
    /// );
    ///
    /// // ...and `result` will receive the output value when it's done
    /// b.add(
    ///     b.value(0).lift2(result, |i, j| { (i, j) })
    /// );
    /// ```
    ///
    pub fn async<SA, A>(&self, root: SA) -> Branch<A> where // NOTE: Needs to be cloneable
        SA: 'static + Signal<A>,
        A: 'static + Clone + Send,
    {
        let v = root.initial();
        let (tx, rx) = channel();
        let pusher = Async::new(Box::new(root), tx);
        self.runners.borrow_mut().push(Box::new(pusher));

        self.listen(v.unwrap(), rx)
    }
}