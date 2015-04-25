use std::marker::*;

use super::super::{Event, Signal, Push, Lift, Lift2, Fold};

pub struct LiftSignal<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send,
{
    pub parent: Box<Signal<A>>,
    pub f: F,
}

impl<F, A, B> Signal<B> for LiftSignal<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send,
{
    fn push_to(self: Box<Self>, target: Option<Box<Push<B>>>) {
        let inner = *self;
        let LiftSignal { parent, f } = inner;

        match target {
            Some(t) => {
                println!("Lift::push_to Some");

                parent.push_to(
                    Some(
                        Box::new(
                            LiftPusher {
                                child: t,
                                f: f,
                                marker: PhantomData,
                            }
                        )
                    )
                );
            },
            None => {
                println!("Lift::push_to None");

                parent.push_to(None)
            },
        }
    }
}

impl<F, A, B> Lift<B> for LiftSignal<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send,
{}
impl<F, A, B, C, SC> Lift2<B, C, SC> for LiftSignal<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send,
{}
impl<F, A, B> Fold<B> for LiftSignal<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send,
{}


struct LiftPusher<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send,
{
    child: Box<Push<B>>,
    f: F,
    marker: PhantomData<A>,
}

impl<F, A, B> Push<A> for LiftPusher<F, A, B> where
    F: 'static + Send + Fn(A) -> B,
    A: 'static + Send,
    B: 'static + Send,
{
    fn push(&mut self, event: Event<A>) {
        let out = match event {
            Event::Changed(a) => {
                println!("LiftPusher handling Event::Changed");
                let b = (self.f)(a);
                Event::Changed(b)
            },
            Event::Unchanged => {
                println!("LiftPusher handling Event::Unchanged");
                Event::Unchanged
            },
            Event::NoOp => {
                println!("LiftPusher handling Event::NoOp");
                Event::NoOp
            },
            Event::Exit => {
                println!("LiftPusher handling Event::NoOp");
                Event::Exit
            },
        };

        self.child.push(out);
    }
}