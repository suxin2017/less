use std::{clone, vec};

use rand::Rng;

fn l<'a>(parent: Vec<Vec<&'a str>>, current: Vec<&'a str>) -> Vec<Vec<&'a str>> {
    let mut r = vec![];
    for l3 in current {
        for l1 in &parent {
            let v: Vec<_> = l1.clone().into_iter().collect();
            let mut rI = v.clone();
            rI.push(l3);
            r.push(rI);
        }
    }

    r
}

fn main() {
    let a = [1,2,3];
    let mut ai = a.iter();
    ai.next();
    let mut a2 = ai.cycle();
    dbg!(a2.next());
    dbg!(a2.next());
    dbg!(a2.next());
    dbg!(a2.next());
    dbg!(a2.next());
}
