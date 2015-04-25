use std::cell::*;
use std::sync::*;
use std::sync::mpsc::*;
use std::thread;
use std::marker::*;

use super::super::Signal;
use super::input::{Input, RunInput, InternalInput, NoOp};
use super::fork::{Run, Fork, Branch};
use super::channel::Channel;

/// `Builder` is used to construct topologies.  
///
/// Basic builder pattern - `Topology::build` accepts a function which takes
/// a state type `T` and a mutable builder.  The builder can be used to create
/// `Channel`s and to `add` nodes to the topology
///
pub struct Builder {
    inputs: RefCell<Vec<Box<RunInput>>>,
    root_signals: RefCell<Vec<Box<Run>>>,
}

impl Builder {
    /// Add a signal to the topology
    ///
    /// Returns a `Branch<A>`, allowing `root` to be used as input more than once
    ///
    pub fn add<A, SA>(&self, root: SA) -> Branch<A> where
        SA: 'static + Signal<A>,
        A: 'static + Clone + Send,
    {
        let fork_txs = Arc::new(Mutex::new(Vec::new()));

        let fork = Fork::new(Box::new(root), fork_txs.clone());

        self.root_signals.borrow_mut().push(Box::new(fork));

        Branch::new(fork_txs, None)
    }

    /// Listen to `source_rx` and push received data into the topology
    ///
    /// All data must enter the topology via a call to `listen`; this function
    /// ensures data syncronization across the topology.  Each listener runs in 
    /// its own thread
    ///
    pub fn listen<A, T>(&self, input: T) -> Channel<A> where
        T: 'static + Input<A> + Send,
        A: 'static + Clone + Send,
    {
        let (tx, rx) = channel();
        let internal_input = InternalInput {
            input: Box::new(input),
            sink_tx: tx,
        };

        self.inputs.borrow_mut().push(Box::new(internal_input));

        Channel::new(rx)
    }
}

/// `Topology<T>` describes a data flow and controls its execution
///
/// If a record of type `T` is passed to `build`, it will be proxied into the
/// builder function as the second argument.  This allows data to be passed from
/// outside the builder's scope into the topology.
///
pub struct Topology {
    builder: Builder,
}

impl Topology {
    /// Construct a topology
    ///
    /// `F` will be called with a `Builder`, which exposes methods for adding
    /// inputs & transformations to the topology
    ///
    pub fn build<T, F>(state: T, f: F) -> Self where 
        F: Fn(&Builder, T),
    {
        let builder = Builder { root_signals: RefCell::new(Vec::new()), inputs: RefCell::new(Vec::new()) };
        f(&builder, state);
        
        Topology { builder: builder }
    }

    /// Run the topology
    ///
    pub fn run(self) {
        let Builder {inputs, root_signals} = self.builder;

        for root_signal in root_signals.into_inner().into_iter() {
            thread::spawn(move || {
                root_signal.run();
            });
        }

        let no_ops = Arc::new(Mutex::new(inputs.borrow().iter().map(|i| i.boxed_no_op()).collect::<Vec<Box<NoOp>>>()));
        for (idx, input) in inputs.into_inner().into_iter().enumerate() {
            let no_ops_i = no_ops.clone();
            thread::spawn(move || {
                input.run(idx, no_ops_i);
            });
        }
    }
}