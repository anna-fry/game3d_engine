// use super::{Marble, Wall};
use crate::geom::*;
use crate::shapes::*;

#[derive(Clone, Copy, Debug)]
pub struct Contact<T: Copy> {
    pub a: T,
    pub b: T,
    pub mtv: Vec3,
}

pub struct Contacts {
    pub wm: Vec<Contact<usize>>,
    pub mm: Vec<Contact<usize>>,
}

impl Contacts {
    pub fn new() -> Self {
        Self {
            wm: vec![],
            mm: vec![],
        }
    }
    fn sort(&mut self) {
        self.wm
            .sort_unstable_by(|a, b| b.mtv.magnitude2().partial_cmp(&a.mtv.magnitude2()).unwrap());
        self.mm
            .sort_unstable_by(|a, b| b.mtv.magnitude2().partial_cmp(&a.mtv.magnitude2()).unwrap());
    }
    fn clear(&mut self) {
        self.wm.clear();
        self.mm.clear();
    }
}

fn restitute(walls: &[Wall], marbles: &mut [Marble], contacts: &mut Contacts) {
    contacts.sort();
    // Lots of marbles on the floor...
    for c in contacts.wm.iter() {
        let a = c.a;
        let b = c.b;
        // Are they still touching?  This way we don't need to track disps or anything
        // at the expense of some extra collision checks
        if let Some(disp) = disp_sphere_plane(&marbles[a].body, &walls[b].body) {
            // We can imagine we're instantaneously applying a
            // velocity change to pop the object just above the floor.
            // marbles[a].body.c += disp;
            // It feels a little weird to be adding displacement (in
            // units) to velocity (in units/frame), but we'll roll
            // with it.  We're not exactly modeling a normal force
            // here but it's something like that.
            marbles[a].velocity += disp ;
            // marbles[a].apply_impulse(disp);
        }
    }
    // That can bump into each other in perfectly elastic collisions!
    for c in contacts.mm.iter() {
        let a = c.a;
        let b = c.b;
        // Just split the difference.  In crowded situations this will
        // cause issues, but those will always be hard to solve with
        // this kind of technique.
        if let Some(disp) = disp_sphere_sphere(&marbles[a].body, &marbles[b].body) {
            marbles[a].body.c -= disp / 2.0;
            marbles[a].velocity -= disp / 2.0;
            marbles[b].body.c += disp / 2.0;
            marbles[b].velocity += disp / 2.0;
            // marbles[a].apply_impulse(-1.0*disp/2.0);
            // marbles[b].apply_impulse(disp/2.0);
        }
    }
}

pub fn update(walls: &[Wall], marbles: &mut [Marble], contacts: &mut Contacts) {
    contacts.clear();
    gather_contacts(walls, marbles, contacts);
    restitute(walls, marbles, contacts);
}

fn gather_contacts(statics: &[Wall], dynamics: &[Marble], into: &mut Contacts) {
    // collide mobiles against mobiles
    for (ai, a) in dynamics.iter().enumerate() {
        for (bi, b) in dynamics[(ai + 1)..].iter().enumerate() {
            let bi = ai + 1 + bi;
            if let Some(disp) = disp_sphere_sphere(&a.body, &b.body) {
                into.mm.push(Contact {
                    a: ai,
                    b: bi,
                    mtv: disp,
                });
            }
        }
    }
    // collide mobiles against walls
    for (bi, b) in statics.iter().enumerate() {
        for (ai, a) in dynamics.iter().enumerate() {
            if let Some(disp) = disp_sphere_plane(&a.body, &b.body) {
                into.wm.push(Contact {
                    a: ai,
                    b: bi,
                    mtv: disp,
                });
            }
        }
    }
}