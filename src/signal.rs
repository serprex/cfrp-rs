use super::{Signal, Lift, Fold, LiftN, InputList, PullInputs, InternalSignal};

impl<A> Signal<A>
{
    /// Apply a pure function `F` to a data source `Signal<A>`, generating a 
    /// transformed output data source `Signal<B>`.
    ///
    /// Other names for this operation include "map" or "collect".  `f` will be
    /// run in `self`'s thread
    ///
    /// Because `F` is assumed to be pure, it will only be evaluated for
    /// new data that has changed since the last observation.  If side-effects are
    /// desired, use `fold` instead.
    ///
    pub fn lift<F, B>(self, f: F) -> Signal<B> where
        F: 'static + Send + Fn(A) -> B,
        A: 'static + Send,
        B: 'static + Send,
    {
        Signal {
            internal_signal: Box::new(
                Lift {
                    parent: self.internal_signal,
                    f: f,
                }
            ),
        }
    }

    pub fn liftn<F, R, B>(self, rest: R, f: F) -> Signal<B> where
        F: 'static + Send + Fn(<<R as InputList<Box<InternalSignal<A>>>>::InputPullers as PullInputs>::Values) -> B,
        R: 'static + Send + InputList<Box<InternalSignal<A>>>,
        B: 'static + Send,
    {
        Signal {
            internal_signal: Box::new(
                LiftN {
                    head: self.internal_signal,
                    rest: rest,
                    f: f,
                }
            ),
        }
    }

    /// Apply a function `F` which uses a data source `Signal<A>` to 
    /// mutate an instance of `B`, generating an output data source `Signal<B>`
    /// containing the mutated value
    ///
    /// Other names for this operation include "reduce" or "inject".  `f` will
    /// be run in `self`'s thread
    ///
    /// Fold is assumed to be impure, therefore the function will be called with
    /// all data upstream of the fold, even if there are no changes in the stream.
    ///
    pub fn foldp<F, B>(self, initial: B, f: F) -> Signal<B> where
        F: 'static + Send + FnMut(&mut B, A),
        A: 'static + Send,
        B: 'static + Send + Clone,
    {
        Signal {
            internal_signal: Box::new(
                Fold {
                    parent: self.internal_signal,
                    f: f,
                    state: initial,
                }
            )
        }
    }
}
