use mlua::prelude::*;

use crate::{event::NoteEvent, tidal::Cycle};

use super::unwrap::{bad_argument_error, note_event_from_value};

// ---------------------------------------------------------------------------------------------

/// Cycle Userdata in bindings
#[derive(Clone, Debug)]
pub struct CycleUserData {
    pub cycle: Cycle,
    pub mappings: Vec<(String, Option<NoteEvent>)>,
    pub mapping_function: Option<LuaOwnedFunction>,
}

impl CycleUserData {
    pub fn from(arg: LuaString, seed: Option<[u8; 32]>) -> LuaResult<Self> {
        // a single value, probably a sequence
        let cycle = Cycle::from(&arg.to_string_lossy(), seed).map_err(LuaError::runtime)?;
        let mappings = Vec::new();
        let mapping_function = None;
        Ok(CycleUserData {
            cycle,
            mappings,
            mapping_function,
        })
    }
}

impl LuaUserData for CycleUserData {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("map", |_lua, this, value: LuaValue| match value {
            LuaValue::Function(func) => {
                let cycle = this.cycle.clone();
                let mappings = Vec::new();
                let mapping_function = Some(func.into_owned());
                Ok(CycleUserData {
                    cycle,
                    mappings,
                    mapping_function,
                })
            }
            LuaValue::Table(table) => {
                let cycle = this.cycle.clone();
                let mut mappings = Vec::new();
                for (k, v) in table.pairs::<LuaValue, LuaValue>().flatten() {
                    mappings.push((k.to_string()?, note_event_from_value(&v, None)?));
                }
                let mapping_function = None;
                Ok(CycleUserData {
                    cycle,
                    mappings,
                    mapping_function,
                })
            }
            _ => Err(bad_argument_error(
                None,
                "map",
                1,
                format!(
                    "map argument must be a table but is a '{}'",
                    value.type_name()
                )
                .as_str(),
            )),
        });
    }
}

// --------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::*;

    use crate::{
        bindings::*,
        event::{cycle::CycleEventIter, new_note, scripted_cycle::ScriptedCycleEventIter},
        Event, EventIter, Note, PulseIterItem,
    };

    fn new_test_engine() -> LuaResult<(Lua, LuaTimeoutHook)> {
        new_test_engine_with_timebase(&BeatTimeBase {
            beats_per_min: 120.0,
            beats_per_bar: 4,
            samples_per_sec: 44100,
        })
    }

    fn new_test_engine_with_timebase(time_base: &BeatTimeBase) -> LuaResult<(Lua, LuaTimeoutHook)> {
        let (mut lua, mut timeout_hook) = new_engine()?;
        register_bindings(&mut lua, &timeout_hook, time_base)?;
        timeout_hook.reset();
        Ok((lua, timeout_hook))
    }

    fn evaluate_cycle_userdata(lua: &Lua, expression: &str) -> LuaResult<CycleUserData> {
        Ok(lua
            .load(expression)
            .eval::<LuaValue>()?
            .as_userdata()
            .ok_or(LuaError::RuntimeError("No user data".to_string()))?
            .borrow::<CycleUserData>()?
            .clone())
    }

    #[test]
    fn parse() -> LuaResult<()> {
        let (lua, _) = new_test_engine()?;

        assert!(evaluate_cycle_userdata(&lua, r#"cycle({})"#).is_err());
        assert!(evaluate_cycle_userdata(&lua, r#"cycle("")"#).is_err());
        assert!(evaluate_cycle_userdata(&lua, r#"cycle("[<")"#).is_err());
        assert!(evaluate_cycle_userdata(&lua, r#"cycle("[c4 e6]")"#).is_ok());

        Ok(())
    }

    #[test]
    fn mappings() -> LuaResult<()> {
        let (lua, _) = new_test_engine()?;

        let mapped_cycle = evaluate_cycle_userdata(
            &lua,
            r#"cycle("a b c"):map({a = "c0", b = 48, c = { key = "c6" }})"#,
        )?;
        assert_eq!(
            mapped_cycle
                .mappings
                .clone()
                .into_iter()
                .collect::<HashMap<_, _>>(),
            HashMap::from([
                ("a".to_string(), new_note(Note::C0)),
                ("b".to_string(), new_note(Note::C4)),
                ("c".to_string(), new_note(Note::C6)),
            ])
        );

        // check if mappings are applied correctly
        let mut event_iter =
            CycleEventIter::new(mapped_cycle.cycle).with_mappings(&mapped_cycle.mappings);
        assert_eq!(
            event_iter
                .run(PulseIterItem::default(), true)
                .map(|events| events.into_iter().map(|e| e.event).collect::<Vec<_>>()),
            Some(vec![
                Event::NoteEvents(vec![new_note(Note::C0)]),
                Event::NoteEvents(vec![new_note(Note::C4)]),
                Event::NoteEvents(vec![new_note(Note::C6)])
            ])
        );

        // check instrument overrides
        let mapped_cycle = evaluate_cycle_userdata(
            &lua,
            r#"cycle("a:1 a:2 a"):map({ a = { key = 48, instrument = 66 } })"#,
        )?;
        let mut event_iter =
            CycleEventIter::new(mapped_cycle.cycle).with_mappings(&mapped_cycle.mappings);
        assert_eq!(
            event_iter
                .run(PulseIterItem::default(), true)
                .map(|events| events.into_iter().map(|e| e.event).collect::<Vec<_>>()),
            Some(vec![
                Event::NoteEvents(vec![new_note((Note::C4, InstrumentId::from(1)))]),
                Event::NoteEvents(vec![new_note((Note::C4, InstrumentId::from(2)))]),
                Event::NoteEvents(vec![new_note((Note::C4, InstrumentId::from(66)))])
            ])
        );
        Ok(())
    }

    #[test]
    fn mapping_functions() -> LuaResult<()> {
        let time_base = BeatTimeBase {
            beats_per_min: 120.0,
            beats_per_bar: 4,
            samples_per_sec: 44100,
        };

        let (lua, timeout_hook) = new_test_engine_with_timebase(&time_base)?;

        let mapped_cycle = evaluate_cycle_userdata(
            &lua,
            r#"
                cycle("wurst a b c"):map(function(context, value) 
                    assert(context.beats_per_min, 120)
                    assert(context.beats_per_bar, 4)
                    assert(context.samples_per_sec, 44100)
                    if value == "wurst" then
                      return "c#4"
                    else
                      return value..4
                    end
                end)"#,
        )?;
        let mapping_callback =
            LuaCallback::with_owned(&lua, mapped_cycle.mapping_function.unwrap().clone())?;
        let mut event_iter = ScriptedCycleEventIter::with_mapping_callback(
            mapped_cycle.cycle,
            &timeout_hook,
            mapping_callback,
            &time_base,
        )?;
        assert_eq!(
            event_iter
                .run(PulseIterItem::default(), true)
                .map(|events| events.into_iter().map(|e| e.event).collect::<Vec<_>>()),
            Some(vec![
                Event::NoteEvents(vec![new_note(Note::Cs4)]),
                Event::NoteEvents(vec![new_note(Note::A4)]),
                Event::NoteEvents(vec![new_note(Note::B4)]),
                Event::NoteEvents(vec![new_note(Note::C4)])
            ])
        );
        Ok(())
    }
}
