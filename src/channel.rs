use std::sync::mpsc::*;

use super::{Channel, Signal, Event};

impl<A> Signal<A> for Channel<A> where
    A: 'static + Send,
{
    fn recv(&mut self) -> Event<A> {
        match self.source_rx.recv() {
            Err(_) => Event::Exit,
            Ok(a) => a,
        }
    }
}
