// use super::{Marble, Wall};
use crate::geom::Vec3;
use crate::physics::{Physics, DT};
use crate::shapes::*;
use crate::geom::*;
const COEFF_R: f32 = 0.5;

#[derive(Clone, Copy, Debug)]
pub struct Contact<T: Copy> {
    pub a: T,
    pub b: T,
    pub mtv: Vec3,
}

#[derive(Debug)]
pub enum CollisionEffect {
    Score,
    WallCollision,
    BallCollision,
    None,
}
#[derive(Debug)]
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

pub struct CollisionDetection {
    contacts: Contacts,
}

impl CollisionDetection {
    pub fn new() -> Self {
        CollisionDetection {
            contacts: Contacts::new(),
        }
    }
    pub fn restitute(&mut self, statics: &[Static], balls: &mut [Ball], physics: &mut [Physics]) {
        self.contacts.sort();
        // Lots of marbles on the floor...

        for c in self.contacts.wm.iter() {
            let a = c.a;
            let b = c.b;
            // Are they still touching?  This way we don't need to track disps or anything
            // at the expense of some extra collision checks
            if let Some(disp) = disp_sphere_plane(&balls[a].body, &statics[b].body) {
                // We can imagine we're instantaneously applying a
                // velocity change to pop the object just above the floor.
                // marbles[a].body.c += disp;
                // It feels a little weird to be adding displacement (in
                // units) to velocity (in units/frame), but we'll roll
                // with it.  We're not exactly modeling a normal force
                // here but it's something like that.
                balls[a].body.c += disp;
                physics[a].momentum += (disp * balls[a].mass * COEFF_R) * DT;
            }
        }
        // That can bump into each other in perfectly elastic collisions!
        for c in self.contacts.mm.iter() {
            let a = c.a;
            let b = c.b;
            // Just split the difference.  In crowded situations this will
            // cause issues, but those will always be hard to solve with
            // this kind of technique.
            if let Some(disp) = disp_sphere_sphere(&balls[a].body, &balls[b].body) {
                let m1 = balls[a].mass;
                let v1 = physics[a].momentum / m1;
                let m2 = balls[b].mass;
                let v2 = physics[b].momentum / m2;

                let v1f = (m1 * v1 + 2.0 * m2 * v2 - m2 * v1) / (m1 + m2);
                let v2f = v1 + v1f - v2;

                let v1r = v1f - v1;
                let v2r = v2f - v2;

                balls[a].body.c -= disp / 2.0;
                physics[a].apply_impulse(COEFF_R * m1 * v1r);
                balls[b].body.c += disp / 2.0;
                physics[b].apply_impulse(COEFF_R * m2 * v2r);
            }
        }
    }

    pub fn update(
        &mut self,
        statics: &[Static],
        balls: &mut [Ball],
        goal: &Goal,
        physics: &mut [Physics],
    ) -> CollisionEffect {
        self.contacts.clear();
        let effect = self.gather_contacts(statics, balls, goal);
        self.restitute(statics, balls, physics);
        effect
    }

    pub fn gather_contacts(
        &mut self,
        statics: &[Static],
        dynamics: &[Ball],
        goal: &Goal,
    ) -> CollisionEffect {
        let mut effect = CollisionEffect::None;
        
        // collide mobiles against mobiles
        for (ai, a) in dynamics.iter().enumerate() {
            for (bi, b) in dynamics[(ai + 1)..].iter().enumerate() {
                let bi = ai + 1 + bi;
                if let Some(disp) = disp_sphere_sphere(&a.body, &b.body) {
                    self.contacts.mm.push(Contact {
                        a: ai,
                        b: bi,
                        mtv: disp,
                    });
                    effect = CollisionEffect::BallCollision;
                }
            }
        }
        // collide mobiles against walls
        for (bi, b) in statics.iter().enumerate() {
            for (ai, a) in dynamics.iter().enumerate() {
                if let Some(disp) = disp_sphere_plane(&a.body, &b.body) {
                    self.contacts.wm.push(Contact {
                        a: ai,
                        b: bi,
                        mtv: disp,
                    });
                    effect = CollisionEffect::WallCollision;
                }
            }
        }

        for a in dynamics.iter() {
            if touching_sphere_box(&a.body, &goal.body) {
                effect = CollisionEffect::Score;
            }
        }

        effect
    }
}
