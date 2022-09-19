use std::{fmt::Display, rc::Rc};

pub mod emitter;
pub mod value;

// -------------------------------------------------------------------------------------------------

/// Sample time value type emitted by the [`Emitter`] trait.
type SampleTime = usize;

/// Id to trigger a specific instruement in an event.
type InstrumentId = usize;
/// Id to trigger a specific parameter in an event.
type ParameterId = usize;

// -------------------------------------------------------------------------------------------------

/// A single event which can be triggered by an [`Emitter`] iterator.
#[derive(Clone, Debug)]
pub enum EmitterEvent {
    NoteEvent {
        instrument: Option<InstrumentId>,
        note: u32,
        velocity: f32,
    },
    ParameterChangeEvent {
        parameter: Option<ParameterId>,
        value: f32,
    },
}
// -------------------------------------------------------------------------------------------------

/// `EmitterValue` is a value which is emitted by the [`Emitter`] trait iter. EmitterValues are
/// iters by themselves, so they may also change over time.
///
/// The most simple example of an `EmitterValue` is a constant note or DSP parameter value change.
/// Being an iterator, this value also then can change over time to e.g. produce note sequences
/// in an arpeggiator or parameter automation.
pub trait EmitterValue: Iterator<Item = Vec<EmitterEvent>> + Display {}

// -------------------------------------------------------------------------------------------------

/// An `Emittor` is an iterator which emits `Option<[`EmitterValue`]>` at a given sample time.
///
/// An Emittor is what triggers events rythmically or periodically in a sequencer, and produces
/// values that happen at a given sample time. Players will use the sample time to schedule the
/// events in the audio stream. When a defined beat-based time-base is used in emitter impls,
/// such emitters may produce beat based values internally too, but those beat times always will
/// render down to sample times.
pub trait Emitter: Iterator<Item = (SampleTime, Option<Vec<EmitterEvent>>)> {
    fn current_value(&self) -> &dyn EmitterValue;
    fn current_sample_time(&self) -> SampleTime;
}

// -------------------------------------------------------------------------------------------------

/// An `EmitterSequence` runs one or more [`Emitter`] at the same time, allowing to form complex
/// sequences of emitters that should be run together.
pub struct EmitterSequence {
    emitters: Vec<Rc<dyn Emitter>>,
}

// -------------------------------------------------------------------------------------------------

impl Display for EmitterEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmitterEvent::NoteEvent {
                instrument,
                note,
                velocity,
            } => f.write_fmt(format_args!(
                "{} {} {}",
                if instrument.is_some() {
                    instrument.unwrap().to_string()
                } else {
                    "NA".to_string()
                },
                note,
                velocity
            )),
            EmitterEvent::ParameterChangeEvent { parameter, value } => f.write_fmt(format_args!(
                "{} {}",
                if parameter.is_some() {
                    parameter.unwrap().to_string()
                } else {
                    "NA".to_string()
                },
                value
            )),
        }
    }
}

// -------------------------------------------------------------------------------------------------

impl EmitterSequence {
    pub fn run(&mut self) {
        for emitter in self.emitters.iter_mut() {
            for (sample_time, events) in Rc::get_mut(emitter).unwrap().take(8) {
                println!(
                    "Time: {:08} - {}",
                    sample_time,
                    match events {
                        Some(events) => events
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<String>>()
                            .join(" | "),
                        None => "---".to_string(),
                    }
                );
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use crate::EmitterSequence;
    use std::rc::Rc;

    use super::emitter::{beat_time::*, beat_time_pattern::*, BeatTimeBase, BeatTimeStep};
    use super::{
        value::{fixed::*, mapped::*},
        EmitterEvent,
    };

    #[test]
    fn test() {
        let time_base = BeatTimeBase {
            beats_per_bar: 4,
            samples_per_sec: 44100,
            beats_per_min: 120.0,
        };

        let note_vector = FixedEmitterValue::new(vec![
            EmitterEvent::NoteEvent {
                instrument: None,
                note: 60,
                velocity: 1.0,
            },
            EmitterEvent::NoteEvent {
                instrument: None,
                note: 64,
                velocity: 1.0,
            },
            EmitterEvent::NoteEvent {
                instrument: None,
                note: 68,
                velocity: 1.0,
            },
        ]);
        let note = MappedEmitterValue::new(
            vec![EmitterEvent::NoteEvent {
                instrument: None,
                note: 60,
                velocity: 1.0,
            }],
            |v| {
                let mut new = v.clone();
                for v in &mut new {
                    match v {
                        EmitterEvent::NoteEvent {
                            instrument: _,
                            note,
                            velocity: _,
                        } => {
                            *note += 1;
                        }
                        EmitterEvent::ParameterChangeEvent {
                            parameter: _,
                            value: _,
                        } => {}
                    }
                }
                new
            },
        );

        let beat_time = BeatTimeEmitter::new(time_base.clone(), BeatTimeStep::Beats(2), note);

        let pattern = BeatTimePatternEmitter::new(
            time_base,
            BeatTimeStep::Beats(2),
            vec![true, false, false, true],
            note_vector,
        );

        let mut sequence = EmitterSequence {
            emitters: vec![Rc::new(beat_time), Rc::new(pattern)],
        };

        sequence.run();
    }
}
