#![no_std]
use core::{borrow::Borrow, ops::Deref};

use nom::{
    character::complete::digit1,
    combinator::{map_res, recognize},
    IResult,
};
#[macro_use]
extern crate alloc;
use alloc::{borrow::{Cow, ToOwned}, string::String};
use alloc::vec::Vec;
fn my_usize(input: &str) -> IResult<&str, usize> {
    map_res(recognize(digit1), str::parse)(input)
}
fn my_u32(input: &str) -> IResult<&str, u32> {
    map_res(recognize(digit1), str::parse)(input)
}

#[repr(transparent)]
pub struct Ident(String);
#[repr(transparent)]
pub struct IdentRef(str);
impl Deref for Ident {
    type Target = IdentRef;

    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self.0.as_str()) }
    }
}
impl Deref for IdentRef {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl IdentRef {
    pub fn parse<'a>(a: &'a str) -> Option<&'a Self> {
        if a.chars().all(|k| k.is_ascii_alphanumeric() || k == '$') {
            Some(unsafe { core::mem::transmute(a) })
        } else {
            None
        }
    }
    pub fn demangle<'a>(&'a self) -> Option<Cow<'a,str>> {
        let mut ch = vec![];
        let a = self.0.split_once("$");
        let Some((a, mut x)) = a else {
            return Some(Cow::Borrowed(&self.0));
        };
        ch.extend(a.chars());
        loop {
            let b;
            let c;
            (x, b) = my_usize(x).ok()?;
            x = x.strip_prefix("c")?;
            (x, c) = my_u32(x).ok()?;
            ch.insert(b, char::from_u32(c)?);
            x = match x.strip_prefix("$") {
                None => return Some(Cow::Owned(ch.into_iter().collect())),
                Some(y) => y,
            }
        }
    }
}
impl Ident {
    pub fn parse(a: String) -> Option<Self> {
        if a.chars().all(|k| k.is_ascii_alphanumeric() || k == '$') {
            Some(Self(a))
        } else {
            None
        }
    }
    pub fn mangle(a: &str) -> Self {
        let mut v = vec![];
        let mut x = a
            .chars()
            .enumerate()
            .filter(|(i, k)| {
                if k.is_ascii_alphanumeric() {
                    return true;
                };
                v.push((*i, *k));
                return false;
            })
            .map(|a| a.1)
            .collect::<String>();
        for (i, v) in v {
            x.extend(format!("${i}c{}", v as u32).chars());
        }
        return Self(x);
    }
}
impl ToOwned for IdentRef {
    type Owned = Ident;

    fn to_owned(&self) -> Self::Owned {
        Ident(self.0.to_owned())
    }
}
impl Borrow<IdentRef> for Ident {
    fn borrow(&self) -> &IdentRef {
        &**self
    }
}


#[cfg(test)]
#[macro_use]
extern crate quickcheck;


#[cfg(test)]
mod tests{
    use alloc::string::String;

    use crate::{Ident, IdentRef};

    quickcheck! {
        fn mangle_works(a: String) -> bool{
            return Ident::mangle(&a).demangle().map(|a|a.into()) == Some(a);
        }
    }
}