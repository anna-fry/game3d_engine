use std::rc::Rc;
use crate::{geom::Rect, model::Material, render::InstanceGroups};

pub struct Letter {
    pub name: char,
    pub position: [f32; 2],
}

impl Letter {
    pub fn draw_letter(& self, igs: &mut InstanceGroups, mat: &Rc<Material>, pos: [f32; 2]) {
        let mut ascii = self.name as u8;
        
        if ascii < 125 {
            ascii -= 32;
            let x = (((ascii as i32 % 16) * 8) as f32) / 128.0;
            let y = (((ascii as i32 / 16) * 8) as f32) / 112.0;

            let tex_rect = Rect {
                x: x,
                y: y,
                w: 0.0625,
                h: 0.0625,
            };

            let position = Rect {
                x: pos[0],
                y: pos[1],
                w: 0.05,
                h: 0.05
            };
            igs.render_2d(&position, &tex_rect, mat);
        } else {
            panic!(
                "invalid letter, ascii was {}, letter attempted to draw was {}",
                ascii, self.name
            );
        }
    }
}

pub struct Sentence {
    pub letters: Vec<Letter>,
    pub position: [f32; 2],
}

impl Sentence {
    pub fn text_to_sentence(text: &str, pos: [f32; 2]) -> Sentence {
        let mut s = Sentence {
            letters: vec![],
            position: pos,
        };
        for c in text.chars() {
            s.letters.push(Letter {
                name: c,
                position: [0.0, 0.0],
            });
        }
        return s;
    }

    pub fn draw_sentence(&self, igs: &mut InstanceGroups, mat: &Rc<Material>) {
        let mut l_pos = self.position;
        for c in &self.letters {
            if c.name == '\n' {
                l_pos[0] = self.position[0];
                l_pos[1] += 0.05;
            } else {
                c.draw_letter(igs, mat, l_pos);
                l_pos[0] += 0.05;
            }
        }
    }
}



