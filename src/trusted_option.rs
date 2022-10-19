#[allow(unused_imports)]
use builtin_macros::*;
#[allow(unused_imports)]
use builtin::*;

verus! {

pub enum Option<T>{
    Some(T),
    None,
}

impl<T> Option<T> {
    pub fn is_none(&self) -> bool {
        match self {
            Option::Some(_) => false,
            Option::None => true,
        }
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub fn peek(&self) -> &T 
        requires
            self.is_some()
    {
        match self {
            Option::Some(val) => val,
            Option::None => panic!(),
        }
    }
}

}

fn main() {}