use std::{cell::RefCell, cmp::Ordering, collections::HashMap};

use either::Either;

use crate::{
    color_names::TurtleColor,
    gui::{ops::TurtleDraw, popup::PopupData, StampCount},
    polygon::TurtleShape,
    ScreenPosition,
};

use super::{types::PopupID, DrawCommand, TurtleGui, TurtleID, TurtleThread};

#[derive(Default, Debug)]
pub struct IndividualTurtle<U> {
    pub cmds: Vec<DrawCommand>,
    pub ops: Vec<TurtleDraw>,
    pub has_new_cmd: bool,
    pub turtle_shape: TurtleShape,
    pub hide_turtle: bool,
    pub(crate) cvt: ConversionInfo,
    pub ui: RefCell<U>,
}

#[derive(Debug, Default)]
pub(crate) struct Progress {
    pub cmd_index: usize, // index into turtle.cmds[]
    pub fraction: f32,    // fractional part completed of the current command being drawn
}

impl Progress {
    pub(crate) fn set_progress(&mut self, cmd_index: usize, fraction: f32) {
        self.cmd_index = cmd_index;
        self.fraction = fraction;
    }

    pub(crate) fn of(cmd_index: usize, fraction: f32) -> Self {
        Self {
            cmd_index,
            fraction,
        }
    }
}

impl PartialEq for Progress {
    fn eq(&self, other: &Self) -> bool {
        self.cmd_index == other.cmd_index && self.fraction.eq(&other.fraction)
    }
}

impl PartialOrd for Progress {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.cmd_index.cmp(&other.cmd_index) {
            Ordering::Less => Some(Ordering::Less),
            Ordering::Equal => self.fraction.partial_cmp(&other.fraction),
            Ordering::Greater => Some(Ordering::Greater),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct ConversionInfo {
    pub last_progress: Progress,
    pub last_fill_point: Option<usize>,

    trunc_pos: Option<usize>,
    pub polygon_start_point: Option<usize>,

    pub cur_path: Vec<(bool, crate::gui::ops::Point)>,
    pub pencolor: TurtleColor,
    pub penwidth: f32,
    pub fillcolor: TurtleColor,
    pub tpos: [f32; 2],
    pub trot: f32,
}

impl ConversionInfo {
    fn new() -> Self {
        Self {
            penwidth: 1.0,
            ..Default::default()
        }
    }

    pub(crate) fn set_trunc_pos(&mut self, pos: usize) {
        if self.trunc_pos.is_none() {
            self.trunc_pos = Some(pos);
        } else if let Some(cur_pos) = self.trunc_pos
            && pos < cur_pos
        {
            self.trunc_pos = Some(pos);
        }
    }

    pub(crate) fn get_trunc_pos(&mut self) -> Option<usize> {
        self.trunc_pos.take()
    }
}

#[derive(Debug)]
pub struct Handler<U, S: TurtleUI> {
    pub last_id: TurtleID,
    pub turtle: HashMap<TurtleID, IndividualTurtle<U>>,
    pub popups: HashMap<PopupID, PopupData>,
    pub title: String,
    pub screen: S,
}

pub trait TurtleUI {
    fn generate_popup(&mut self, popupdata: &PopupData) -> PopupID;
    fn resize(&mut self, width: isize, height: isize);
    fn set_bg_color(&mut self, bgcolor: TurtleColor);
}

impl<T: Default, U: Default + TurtleUI> TurtleGui for Handler<T, U> {
    fn convert(&mut self, pct: f32, id: &TurtleID) {
        if let Some(turtle) = self.turtle.get_mut(id) {
            if turtle.cmds.is_empty() {
                return;
            }

            crate::gui::ops::TurtleDraw::convert(pct, turtle);
        }
    }

    fn set_title(&mut self, title: String) {
        self.title = title;
    }

    fn clearscreen(&mut self) {
        let id0 = TurtleID::new(0);

        self.turtle.retain(|&k, _| k == id0);

        self.turtle.entry(id0).and_modify(|t| {
            *t = IndividualTurtle::<T> {
                has_new_cmd: true,
                cvt: ConversionInfo::new(),
                ..Default::default()
            };
        });
    }

    fn new_turtle(&mut self) -> TurtleID {
        let id = self.last_id.get();

        self.turtle.insert(
            id,
            IndividualTurtle {
                has_new_cmd: true,
                cvt: ConversionInfo::new(),
                ..Default::default()
            },
        );
        id
    }

    fn shut_down(&mut self) {
        std::process::exit(0);
    }

    fn clear_turtle(&mut self, turtle: TurtleID) {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        turtle.cmds.clear();
        turtle.has_new_cmd = true;
        turtle.cvt = ConversionInfo::new();
    }

    fn set_shape(&mut self, turtle: TurtleID, shape: TurtleShape) {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        turtle.turtle_shape = shape;
        turtle.has_new_cmd = true;
    }

    fn stamp(&mut self, turtle: TurtleID, pos: ScreenPosition<f32>, angle: f32) -> usize {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        turtle.cmds.push(DrawCommand::DrawPolyAt(
            turtle.turtle_shape.poly[0].polygon.clone(),
            pos,
            angle,
        ));
        turtle.cmds.len() - 1
    }

    fn clear_stamp(&mut self, turtle: TurtleID, stamp: usize) {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        assert!(matches!(
            turtle.cmds[stamp],
            DrawCommand::DrawPolyAt(_, _, _)
        ));
        turtle.cmds[stamp] = DrawCommand::Filler;
        turtle.has_new_cmd = true;
    }

    fn clear_stamps(&mut self, turtle: TurtleID, count: StampCount) {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        let all = turtle.cmds.len();
        let (mut iter, mut count) = match count {
            StampCount::Forward(count) => (Either::Right(turtle.cmds.iter_mut()), count),
            StampCount::Reverse(count) => (Either::Left(turtle.cmds.iter_mut().rev()), count),
            StampCount::All => (Either::Right(turtle.cmds.iter_mut()), all),
        };

        while count > 0 {
            if let Some(cmd) = iter.next() {
                if matches!(cmd, DrawCommand::DrawPolyAt(_, _, _)) {
                    count -= 1;
                    *cmd = DrawCommand::Filler;
                }
            } else {
                break;
            }
        }

        turtle.has_new_cmd = true;
    }

    fn get_turtle_shape_name(&mut self, turtle: TurtleID) -> String {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        turtle.turtle_shape.name.clone()
    }

    fn append_command(&mut self, turtle: TurtleID, cmd: DrawCommand) {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        turtle.cmds.push(cmd);
        turtle.has_new_cmd = true;
    }

    fn get_position(&self, turtle: TurtleID) -> usize {
        self.turtle[&turtle].cmds.len()
    }

    fn fill_polygon(&mut self, turtle: TurtleID, cmd: DrawCommand, index: usize) {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        turtle.has_new_cmd = true;
        turtle.cmds[index] = cmd;
        turtle.cmds.push(DrawCommand::Filled(index));
    }

    fn undo_count(&self, turtle: TurtleID) -> usize {
        self.turtle.get(&turtle).expect("missing turtle").cmds.len()
    }

    fn undo(&mut self, turtle: TurtleID) {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        turtle.has_new_cmd = true;
    }

    fn pop(&mut self, turtle: TurtleID) -> Option<DrawCommand> {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        let cmd = turtle.cmds.pop();

        if let Some(DrawCommand::Filled(index)) = &cmd {
            turtle.cmds[*index] = DrawCommand::Filler;
        }

        cmd
    }

    fn numinput(&mut self, turtle: TurtleID, thread: TurtleThread, title: &str, prompt: &str) {
        let popupdata = PopupData::num_input(title, prompt, turtle, thread);
        let id = self.screen.generate_popup(&popupdata);
        self.popups.insert(id, popupdata);
    }

    fn textinput(&mut self, turtle: TurtleID, thread: TurtleThread, title: &str, prompt: &str) {
        let popupdata = PopupData::text_input(title, prompt, turtle, thread);
        let id = self.screen.generate_popup(&popupdata);
        self.popups.insert(id, popupdata);
    }

    fn bgcolor(&mut self, color: TurtleColor) {
        self.screen.set_bg_color(color);
    }

    fn resize(&mut self, _turtle: TurtleID, _thread: TurtleThread, width: isize, height: isize) {
        self.screen.resize(width, height);
    }

    fn set_visible(&mut self, turtle: TurtleID, visible: bool) {
        let turtle = self.turtle.get_mut(&turtle).expect("missing turtle");
        turtle.hide_turtle = !visible;
        turtle.has_new_cmd = true;
    }

    fn is_visible(&self, turtle: TurtleID) -> bool {
        let turtle = self.turtle.get(&turtle).expect("missing turtle");
        !turtle.hide_turtle
    }
}
