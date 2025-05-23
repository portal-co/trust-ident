#![no_std]
use core::{borrow::Borrow, cmp::Ordering, fmt::{Debug, Display, Formatter}, marker::PhantomData, ops::Deref};

use duplicate::duplicate_item;
use nom::{
    character::complete::digit1,
    combinator::{map_res, recognize},
    IResult, Parser,
};
#[macro_use]
extern crate alloc;
use alloc::vec::Vec;
use alloc::{
    borrow::{Cow, ToOwned},
    string::String,
};
fn my_usize(input: &str) -> IResult<&str, usize> {
    map_res(recognize(digit1), str::parse).parse(input)
}
fn my_u32(input: &str) -> IResult<&str, u32> {
    map_res(recognize(digit1), str::parse).parse(input)
}

pub trait Cfg {
    fn valid(ch: char) -> bool;
    const EMBED: &'static str;
    const SEP: &'static str;
}
pub struct CCfg {}
impl Cfg for CCfg {
    fn valid(k: char) -> bool {
        k.is_ascii_alphanumeric() || k == '$'
    }

    const EMBED: &'static str = "$";

    const SEP: &'static str = "c";
}

#[repr(transparent)]
pub struct Ident<C: Cfg>(String, PhantomData<fn(&C) -> &C>);
impl<C: Cfg> Clone for Ident<C>{
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}
#[repr(transparent)]
pub struct IdentRef<C: Cfg>(PhantomData<fn(&C) -> &C>, str);
impl<C: Cfg> Deref for Ident<C> {
    type Target = IdentRef<C>;

    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self.0.as_str()) }
    }
}
impl<C: Cfg> Deref for IdentRef<C> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}
#[duplicate_item(
    typ;
    [Ident];
    [IdentRef];
)]
const _: () = {
    impl<C: Cfg> PartialEq for typ<C>{
        fn eq(&self, other: &Self) -> bool{
            return self.as_str() == other.as_str();
        }
    }
    impl<C: Cfg> Eq for typ<C>{

    }
    impl<C: Cfg> PartialOrd for typ<C>{
        fn partial_cmp(&self, other: &Self) -> Option<Ordering>{
            self.as_str().partial_cmp(other.as_str())
        }
    }
    impl<C: Cfg> Ord for typ<C>{
        fn cmp(&self, other: &Self) -> Ordering{
            self.as_str().cmp(other.as_str())
        }
    }
    impl<C: Cfg> Debug for typ<C>{
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error>{
            <str as Debug>::fmt(self.as_str(),f)
        }   
    }
    impl<C: Cfg> Display for typ<C>{
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error>{
            <str as Display>::fmt(self.as_str(),f)
        }   
    }
};

impl<C: Cfg> IdentRef<C> {
    pub fn as_str<'a>(&'a self) -> &'a str{
        self
    }
    pub fn parse<'a>(a: &'a str) -> Option<&'a Self> {
        if a.chars().all(|k| C::valid(k)) {
            Some(unsafe { core::mem::transmute(a) })
        } else {
            None
        }
    }
    pub fn demangle<'a>(&'a self) -> Option<Cow<'a, str>> {
        let mut ch = vec![];
        let a = self.1.split_once(C::EMBED);
        let Some((a, mut x)) = a else {
            return Some(Cow::Borrowed(&self.1));
        };
        ch.extend(a.chars());
        loop {
            let b;
            let c;
            (x, b) = my_usize(x).ok()?;
            x = x.strip_prefix(C::SEP)?;
            (x, c) = my_u32(x).ok()?;
            ch.insert(b, char::from_u32(c)?);
            x = match x.strip_prefix(C::EMBED) {
                None => return Some(Cow::Owned(ch.into_iter().collect())),
                Some(y) => y,
            }
        }
    }
}
impl<C: Cfg> Ident<C> {
    pub fn parse(a: String) -> Option<Self> {
        if a.chars().all(|k| C::valid(k)) {
            Some(Self(a, PhantomData))
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
                if C::valid(*k) && C::EMBED.chars().all(|e| e != *k) {
                    return true;
                };
                v.push((*i, *k));
                return false;
            })
            .map(|a| a.1)
            .collect::<String>();
        for (i, v) in v {
            x.extend(format!("{}{i}{}{}", C::EMBED, C::SEP, v as u32).chars());
        }
        return Self(x, PhantomData);
    }
}
impl<C: Cfg> ToOwned for IdentRef<C> {
    type Owned = Ident<C>;

    fn to_owned(&self) -> Self::Owned {
        Ident(self.1.to_owned(), PhantomData)
    }
}
impl<C: Cfg> Borrow<IdentRef<C>> for Ident<C> {
    fn borrow(&self) -> &IdentRef<C> {
        &**self
    }
}

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use crate::{CCfg, Ident, IdentRef};

    quickcheck! {
        fn mangle_works(a: String) -> bool{
            return Ident::<CCfg>::mangle(&a).demangle().map(|a|a.into()) == Some(a);
        }
    }
}
