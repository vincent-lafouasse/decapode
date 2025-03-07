#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::collections::HashSet;

use egui::Key;

mod event;
mod math;
mod midi;
mod synth;
mod tuner;
mod wavetable;

use crate::event::Event;
use crate::midi::MidiNote;
use crate::synth::Synth;
use crate::wavetable::WavetableKind;

struct App {
    synth: Synth,
    pressed_keys: HashSet<egui::Key>,
    root_note: MidiNote,
    current_wavetable: WavetableKind,
    master_volume: f32,
    attack_ms: u16,
    release_ms: u16,
}

impl Default for App {
    fn default() -> Self {
        let synth = Synth::new();
        let pressed_keys: HashSet<egui::Key> = HashSet::new();
        let root_note = MidiNote::c(2);
        let current_wavetable = WavetableKind::Triangle;

        Self {
            synth,
            pressed_keys,
            root_note,
            current_wavetable,
            master_volume: 0.7,
            attack_ms: 300,
            release_ms: 200,
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "decapode",
        options,
        Box::new(|_cc| Ok(Box::<App>::default())),
    )
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ctx.input(|i| i.viewport().close_requested()) {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }

            ui.heading("Oscillator");
            ui.vertical(|ui| {
                let kind = WavetableKind::Triangle;
                if ui
                    .radio_value(&mut self.current_wavetable, kind, format!("{kind}"))
                    .clicked()
                {
                    self.synth.send_event(Event::ChangeOscillator(kind));
                }
                let kind = WavetableKind::Square;
                if ui
                    .radio_value(&mut self.current_wavetable, kind, format!("{kind}"))
                    .clicked()
                {
                    self.synth.send_event(Event::ChangeOscillator(kind));
                }
            });
            ui.end_row();

            if ui
                .add(egui::Slider::new(&mut self.master_volume, 0.0..=1.0).text("Master"))
                .dragged()
            {
                self.synth.send_event(Event::SetMaster(self.master_volume));
            }

            if ui
                .add(
                    egui::Slider::new(&mut self.attack_ms, 5..=10000)
                        .logarithmic(true)
                        .text("Attack (ms)"),
                )
                .dragged()
            {
                self.synth.send_event(Event::SetAttackMs(self.attack_ms));
            }
            if ui
                .add(
                    egui::Slider::new(&mut self.release_ms, 5..=10000)
                        .logarithmic(true)
                        .text("Release (ms)"),
                )
                .dragged()
            {
                self.synth.send_event(Event::SetReleaseMs(self.release_ms));
            }

            let events = ui.ctx().input(|i| i.events.clone());
            'event_loop: for event in &events {
                match event {
                    egui::Event::Key {
                        key: Key::Escape,
                        pressed: false,
                        ..
                    } => ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close),
                    egui::Event::Key {
                        key: Key::Z,
                        pressed: false,
                        ..
                    } => self.synth.send_event(Event::OctaveDown),
                    egui::Event::Key {
                        key: Key::X,
                        pressed: false,
                        ..
                    } => self.synth.send_event(Event::OctaveUp),
                    egui::Event::Key { key, pressed, .. } => {
                        let note = keymap(key, self.root_note);
                        if note.is_none() {
                            continue 'event_loop;
                        }
                        let note = note.unwrap();
                        match pressed {
                            // NoteOn
                            true => {
                                if !self.pressed_keys.contains(key) {
                                    self.synth.send_event(Event::NoteOn(note));
                                    self.pressed_keys.insert(*key);
                                }
                            }
                            // NoteOff
                            false => {
                                if self.pressed_keys.contains(key) {
                                    self.synth.send_event(Event::NoteOff(note));
                                    self.pressed_keys.remove(key);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
    }
}
fn keymap(keycode: &Key, root: MidiNote) -> Option<MidiNote> {
    match keycode {
        // second row is white keys
        Key::A => Some(root),
        Key::S => Some(root.offset_up(2)),
        Key::D => Some(root.offset_up(4)),
        Key::F => Some(root.offset_up(5)),
        Key::G => Some(root.offset_up(7)),
        Key::H => Some(root.offset_up(9)),
        Key::J => Some(root.offset_up(11)),
        Key::K => Some(root.offset_up(12)),
        Key::L => Some(root.offset_up(14)),
        Key::Semicolon => Some(root.offset_up(16)),
        Key::Quote => Some(root.offset_up(17)),
        // first row is black keys
        Key::W => Some(root.offset_up(1)),
        Key::E => Some(root.offset_up(3)),
        Key::T => Some(root.offset_up(6)),
        Key::Y => Some(root.offset_up(8)),
        Key::U => Some(root.offset_up(10)),
        Key::I => Some(root.offset_up(13)),
        Key::O => Some(root.offset_up(15)),
        _ => None,
    }
}
