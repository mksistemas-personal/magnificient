use std::marker::PhantomData;

pub struct Context<S, D> {
    pub data: D,
    _state: PhantomData<S>,
}

impl<S, D> Context<S, D> {
    pub fn new(data: D) -> Self {
        Self {
            data,
            _state: PhantomData,
        }
    }

    pub fn transition<S2, D2>(self, data: D2) -> Context<S2, D2> {
        Context {
            data,
            _state: PhantomData,
        }
    }

    pub fn map<S2, D2, F>(self, operation: F) -> Context<S2, D2>
    where
        F: FnOnce(D) -> D2,
    {
        Context {
            data: operation(self.data),

            _state: PhantomData,
        }
    }
}
