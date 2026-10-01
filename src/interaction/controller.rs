use gilrs::{Axis, Button, EventType, GamepadId, Gilrs};

#[derive(Clone, Copy, Default)]
struct Buttons {
    south: bool,
    east: bool,
    west: bool,
    north: bool,
    start: bool,
    dpad_left: bool,
    dpad_right: bool,
    dpad_up: bool,
    dpad_down: bool,
}

pub struct ControllerInput {
    gilrs: Option<Gilrs>,
    active_id: Option<GamepadId>,
    previous: Buttons,
    current: Buttons,
    left_stick: (f32, f32),
    right_stick: (f32, f32),
}

impl ControllerInput {
    pub fn new() -> Self {
        let gilrs = Gilrs::new().ok();
        Self {
            gilrs,
            active_id: None,
            previous: Buttons::default(),
            current: Buttons::default(),
            left_stick: (0.0, 0.0),
            right_stick: (0.0, 0.0),
        }
    }

    pub fn update(&mut self) {
        let Some(gilrs) = &mut self.gilrs else { return; };
        while let Some(event) = gilrs.next_event() {
            match event.event {
                EventType::Connected => self.active_id = Some(event.id),
                EventType::Disconnected if self.active_id == Some(event.id) => self.active_id = None,
                _ => {}
            }
        }
        if self.active_id.is_none() {
            self.active_id = gilrs.gamepads().next().map(|(id, _)| id);
        }
        self.previous = self.current;
        let Some(id) = self.active_id else {
            self.current = Buttons::default();
            self.left_stick = (0.0, 0.0);
            self.right_stick = (0.0, 0.0);
            return;
        };
        let gamepad = gilrs.gamepad(id);
        self.current = Buttons {
            south: gamepad.is_pressed(Button::South),
            east: gamepad.is_pressed(Button::East),
            west: gamepad.is_pressed(Button::West),
            north: gamepad.is_pressed(Button::North),
            start: gamepad.is_pressed(Button::Start),
            dpad_left: gamepad.is_pressed(Button::DPadLeft),
            dpad_right: gamepad.is_pressed(Button::DPadRight),
            dpad_up: gamepad.is_pressed(Button::DPadUp),
            dpad_down: gamepad.is_pressed(Button::DPadDown),
        };
        self.left_stick = (deadzone(gamepad.value(Axis::LeftStickX)), deadzone(gamepad.value(Axis::LeftStickY)));
        self.right_stick = (deadzone(gamepad.value(Axis::RightStickX)), deadzone(gamepad.value(Axis::RightStickY)));
    }

    pub fn connected(&self) -> bool { self.active_id.is_some() }
    pub fn confirm_pressed(&self) -> bool { self.current.south && !self.previous.south }
    pub fn back_pressed(&self) -> bool { self.current.east && !self.previous.east }
    pub fn eye_pressed(&self) -> bool { self.current.north && !self.previous.north }
    pub fn pause_pressed(&self) -> bool { self.current.start && !self.previous.start }
    pub fn rotate_pressed(&self) -> bool { self.current.west && !self.previous.west }
    pub fn dpad_left(&self) -> bool { self.current.dpad_left }
    pub fn dpad_right(&self) -> bool { self.current.dpad_right }
    pub fn dpad_up(&self) -> bool { self.current.dpad_up }
    pub fn dpad_down(&self) -> bool { self.current.dpad_down }
    pub fn left_stick(&self) -> (f32, f32) { self.left_stick }
    pub fn right_stick(&self) -> (f32, f32) { self.right_stick }
}

fn deadzone(value: f32) -> f32 {
    if value.abs() >= 0.18 { value } else { 0.0 }
}
