use std::sync::*;
use std::sync::mpsc::*;

use super::super::{Event};

pub trait NoOp: Send {
    fn send_no_change(&self) -> bool;
    fn send_exit(&self);
}

pub trait RunInput: Send {
    fn run(mut self: Box<Self>, usize, Arc<Mutex<Vec<Box<NoOp>>>>);
    fn boxed_no_op(&self) -> Box<NoOp>;
}

pub struct ReceiverInput<A> {
    rx: Receiver<A>,
    tx: SyncSender<Event<A>>,
}

impl<A> ReceiverInput<A> {
    pub fn new(rx: Receiver<A>, tx: SyncSender<Event<A>>) -> ReceiverInput<A> {
        ReceiverInput {
            rx: rx,
            tx: tx,
        }
    }
}

impl<A> RunInput for ReceiverInput<A> where
    A: 'static + Send + Clone,
{
    fn boxed_no_op(&self) -> Box<NoOp> {
        Box::new(self.tx.clone())
    }

    fn run(self: Box<Self>, idx: usize, txs: Arc<Mutex<Vec<Box<NoOp>>>>) {
        let inner = *self;
        let ReceiverInput {rx, tx} = inner;

        loop {
            match rx.recv() {
                Ok(ref a) => {
                   for (i, no_op_tx) in txs.lock().unwrap().iter().enumerate() {
                       if i == idx {
                           match tx.send(Event::Changed(a.clone())) {
                               Err(_) => return,
                               _ => {},
                           }
                       } else {
                           if no_op_tx.send_no_change() { return }
                       }
                   }
                },
                Err(_) => {
                    for no_op_tx in txs.lock().unwrap().iter() {
                        no_op_tx.send_exit();
                    }
                    return
                },
            }
        }
    }
}

pub struct ValueInput<A> where
    A: Send,
{
    value: A,
    tx: SyncSender<Event<A>>,
}

impl<A> ValueInput<A> where
    A: Send,
{
    pub fn new(value: A, tx: SyncSender<Event<A>>) -> ValueInput<A> {
        ValueInput {
            value: value,
            tx: tx,
        }
    }
}

impl<A> RunInput for ValueInput<A> where
    A: 'static + Send,
{
    fn boxed_no_op(&self) -> Box<NoOp> {
        Box::new(self.tx.clone())
    }

    fn run(mut self: Box<Self>, idx: usize, txs: Arc<Mutex<Vec<Box<NoOp>>>>) {
        let inner = *self;
        let ValueInput {value, tx} = inner;

        tx.send(Event::Changed(value));

        loop {
            match tx.send(Event::Unchanged) {
                Err(_) => return,
                _ => {},
            }
        }
    }
}



impl<A> NoOp for SyncSender<Event<A>> where
    A: Send
{
    fn send_no_change(&self) -> bool {
        match self.send(Event::Unchanged) {
            Err(_) => true,
            _ => false,
        }
    }

    fn send_exit(&self) {
        self.send(Event::Exit).unwrap();
    }
}