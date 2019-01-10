// encoder.rs
//
// Copyright (c) 2019 Minnesota Department of Transportation
//
use std::vec::Vec;

use crate::geom::{Transform,Vec2};

#[derive(Clone,Debug)]
enum Command {
    MoveTo = 1,
    LineTo = 2,
    ClosePath = 7,
}

struct CommandInt {
    id: Command,
    count: u32,
}

struct ParamInt {
    value: i32,
}

#[derive(Clone,Debug)]
pub enum GeomType {
    Point,
    Linestring,
    Polygon,
}

pub struct GeomEncoder {
    geom_tp: GeomType,
    transform: Transform,
    x: i32,
    y: i32,
    cmd_offset: usize,
    count: u32,
    data: Vec<u32>,
}

impl CommandInt {
    fn new(id: Command, count: u32) -> Self {
        CommandInt { id, count }
    }
    fn encode(&self) -> u32 {
        ((self.id.clone() as u32) & 0x7) | (self.count << 3)
    }
}

impl ParamInt {
    fn new(value: i32) -> Self {
        ParamInt { value }
    }
    fn encode(&self) -> u32 {
        ((self.value << 1) ^ (self.value >> 31)) as u32
    }
}

impl GeomEncoder {
    pub fn new(geom_tp: GeomType, transform: Transform) -> Self {
        GeomEncoder {
            geom_tp,
            transform,
            x: 0,
            y: 0,
            count: 0,
            cmd_offset: 0,
            data: vec!(),
        }
    }

    pub(crate) fn geom_type(&self) -> GeomType {
        self.geom_tp.clone()
    }

    fn command(&mut self, cmd: Command, count: u32) {
        self.cmd_offset = self.data.len();
        debug!("command: {:?}", &cmd);
        self.data.push(CommandInt::new(cmd, count).encode());
    }

    fn set_command(&mut self, cmd: Command, count: u32) {
        let off = self.cmd_offset;
        self.data[off] = CommandInt::new(cmd, count).encode();
    }

    fn encode_point(&mut self, x: f64, y: f64) {
        let p = self.transform * Vec2::new(x, y);
        let x = p.x as i32;
        let y = p.y as i32;
        self.data.push(ParamInt::new(x.saturating_sub(self.x)).encode());
        self.data.push(ParamInt::new(y.saturating_sub(self.y)).encode());
        debug!("point: {},{}", x, y);
        self.x = x;
        self.y = y;
    }

    pub fn add_point(&mut self, x: f64, y: f64) {
        match self.geom_tp {
            GeomType::Point => {
                if self.count == 0 {
                    self.command(Command::MoveTo, 1);
                }
            },
            GeomType::Linestring => {
                match self.count {
                    0 => self.command(Command::MoveTo, 1),
                    1 => self.command(Command::LineTo, 1),
                    _ => (),
                }
            },
            GeomType::Polygon => {
                match self.count {
                    0 => self.command(Command::MoveTo, 1),
                    1 => self.command(Command::LineTo, 1),
                    _ => (),
                }
            },
        }
        self.encode_point(x, y);
        self.count += 1;
    }

    pub fn complete_geom(&mut self) {
        match self.geom_tp {
            GeomType::Point => (),
            GeomType::Linestring => {
                if self.count > 1 {
                    self.set_command(Command::LineTo, self.count - 1);
                }
                self.count = 0;
            },
            GeomType::Polygon => {
                if self.count > 1 {
                    self.set_command(Command::LineTo, self.count - 1);
                    self.command(Command::ClosePath, 1);
                }
                self.count = 0;
            },
        }
    }

    pub(crate) fn to_vec(mut self) -> Vec<u32> {
        if let GeomType::Point = self.geom_tp {
            if self.count > 1 {
                self.set_command(Command::MoveTo, self.count);
            }
        } else {
            self.complete_geom();
        }
        self.data
    }
}
