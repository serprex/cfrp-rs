use std::cell::*;
use std::sync::*;
use std::sync::mpsc::*;
use std::thread::spawn;
use std::marker::*;

use super::input::{Input, CoordinatedInput, NoOp};
use super::{Signal, Run, Fork, Branch, Channel};

/// `Builder` is used to construct topologies.  
///
/// Basic builder pattern - `Topology::build` accepts a function which takes
/// a state type `T` and a mutable builder.  The builder can be used to create
/// `Channel`s and to `add` nodes to the topology
///
pub struct Builder {
    inputs: RefCell<Vec<Box<CoordinatedInput>>>,
    root_signals: RefCell<Vec<Box<Run>>>,
}

impl Builder {
    /// Add a signal to the topology
    ///
    /// Returns a `Branch<A>`, allowing `root` to be used as input more than once
    ///
    pub fn add<A>(&self, root: Box<Signal<A> + Send>) -> Box<Branch<A>> where
        A: 'static + Clone + Send,
    {
        let (tx, rx) = channel();
        let fork_txs = Arc::new(Mutex::new(vec![tx]));

        let fork = Fork::new(root, fork_txs.clone());

        self.root_signals.borrow_mut().push(Box::new(fork));

        Box::new(Branch::new(fork_txs, rx))
    }

    /// Listen to `source_rx` and push received data into the topology
    ///
    /// All data entering a topology must originate in a channel; channels ensure
    /// data syncronization across the topology.  Each channel runs in its own 
    /// thread
    ///
    pub fn channel<A>(&self, source_rx: Receiver<A>) -> Box<Signal<A>> where
        A: 'static + Clone + Send,
    {
        let (tx, rx) = channel();
        let input = Input::new(source_rx, tx);

        self.inputs.borrow_mut().push(Box::new(input));

        Box::new(Channel::new(rx))
    }
}

/// `Topology<T>` describes a data flow and controls its execution
///
pub struct Topology<T> {
    builder: Builder,
    marker: PhantomData<T>,
}

impl<T> Topology<T> {
    /// Construct a topology
    ///
    /// `F` will be called with a `Builder`, which exposes methods for adding
    /// inputs & transformations to the topology
    ///
    pub fn build<F>(state: T, f: F) -> Self where 
        F: Fn(&Builder, T),
    {
        let builder = Builder { root_signals: RefCell::new(Vec::new()), inputs: RefCell::new(Vec::new()) };
        f(&builder, state);
        
        Topology { builder: builder, marker: PhantomData }
    }

    /// Run the topology
    ///
    pub fn run(self) {
        let Builder {inputs, root_signals} = self.builder;

        for root_signal in root_signals.into_inner().into_iter() {
            spawn(move || {
                root_signal.run();
            });
        }

        let no_ops = Arc::new(Mutex::new(inputs.borrow().iter().map(|i| i.boxed_no_op()).collect::<Vec<Box<NoOp>>>()));
        for (idx, input) in inputs.into_inner().into_iter().enumerate() {
            let no_ops_i = no_ops.clone();
            spawn(move || {
                input.run(idx, no_ops_i);
            });
        }
    }
}
