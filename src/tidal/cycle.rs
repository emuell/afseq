use core::fmt::Display;

use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

use rand::{thread_rng, Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use fraction::{Fraction, One, Zero};

use crate::pattern::euclidean::euclidean;

// -------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
enum Step {
    Single(Single),
    Alternating(Alternating),
    Subdivision(Subdivision),
    Polymeter(Polymeter),
    Stack(Stack),
    Choices(Choices),
    Expression(Expression),
    Bjorklund(Bjorklund),
    Repeat,
}

impl Step {
    fn parse_single(single: Pair<Rule>) -> Result<Step, String> {
        match single.into_inner().next() {
            Some(pair) => {
                let string = pair.as_str().to_string();
                let value = Value::parse(pair)?;
                Ok(Step::Single(Single { string, value }))
            }
            None => Err("empty single".to_string()),
        }
    }

    // recursively reset the step to its initial state
    fn reset(&mut self) {
        match self {
            Step::Alternating(a) => {
                a.current = 0;
            }
            Step::Polymeter(pm) => {
                pm.offset = 0;
            }
            _ => (),
        }
        for step in self.inner_steps_mut() {
            step.reset()
        }
    }

    #[allow(dead_code)]
    fn inner_steps(&self) -> Vec<&Step> {
        match self {
            Step::Repeat => vec![],
            Step::Single(_s) => vec![],
            Step::Alternating(a) => a.steps.iter().collect(),
            Step::Polymeter(pm) => pm.steps.iter().collect(),
            Step::Subdivision(sd) => sd.steps.iter().collect(),
            Step::Choices(cs) => cs.choices.iter().collect(),
            Step::Stack(st) => st.stack.iter().collect(),
            Step::Expression(e) => vec![&e.left, &e.right],
            Step::Bjorklund(b) => {
                if let Some(rotation) = &b.rotation {
                    vec![&b.left, &b.steps, &b.pulses, &**rotation]
                } else {
                    vec![&b.left, &b.steps, &b.pulses]
                }
            }
        }
    }

    fn inner_steps_mut(&mut self) -> Vec<&mut Step> {
        match self {
            Step::Repeat => vec![],
            Step::Single(_s) => vec![],
            Step::Alternating(a) => a.steps.iter_mut().collect(),
            Step::Polymeter(pm) => pm.steps.iter_mut().collect(),
            Step::Subdivision(sd) => sd.steps.iter_mut().collect(),
            Step::Choices(cs) => cs.choices.iter_mut().collect(),
            Step::Stack(st) => st.stack.iter_mut().collect(),
            Step::Expression(e) => vec![&mut e.left, &mut e.right],
            Step::Bjorklund(b) => {
                if let Some(rotation) = &mut b.rotation {
                    vec![&mut b.left, &mut b.steps, &mut b.pulses, &mut **rotation]
                } else {
                    vec![&mut b.left, &mut b.steps, &mut b.pulses]
                }
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Single {
    value: Value,
    string: String,
}

impl Single {
    fn default() -> Self {
        Single {
            value: Value::Rest,
            string: String::from("~"),
        }
    }
    fn to_integer(&self) -> Option<i32> {
        match &self.value {
            Value::Rest => None,
            Value::Hold => None,
            Value::Name(_n) => None,
            Value::Integer(i) => Some(*i),
            Value::Float(f) => Some(*f as i32),
            Value::Pitch(n) => Some(n.note as i32),
        }
    }
    fn to_target(&self) -> Target {
        match &self.value {
            Value::Rest => Target::None,
            Value::Hold => Target::None,
            Value::Name(n) => Target::Name(n.clone()),
            Value::Integer(i) => Target::Index(*i),
            Value::Float(f) => Target::Index(*f as i32),
            Value::Pitch(_n) => Target::Name(self.string.clone()), // TODO might not be the best conversion idea
        }
    }
    fn to_chance(&self) -> Option<f64> {
        match &self.value {
            Value::Rest => None,
            Value::Hold => None,
            Value::Name(_n) => None,
            Value::Integer(i) => Some((*i as f64).clamp(0.0, 100.0) / 100.0),
            Value::Float(f) => Some(f.clamp(0.0, 1.0)),
            Value::Pitch(n) => Some((n.note as f64).clamp(0.0, 128.0) / 128.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Alternating {
    current: usize,
    steps: Vec<Step>,
}

#[derive(Clone, Debug, PartialEq)]
struct Subdivision {
    steps: Vec<Step>,
}

#[derive(Clone, Debug, PartialEq)]
struct Polymeter {
    count: usize,
    offset: usize,
    steps: Vec<Step>,
}

#[derive(Clone, Debug, PartialEq)]
struct Choices {
    choices: Vec<Step>,
}

#[derive(Clone, Debug, PartialEq)]
struct Stack {
    stack: Vec<Step>,
}

#[derive(Clone, Debug, PartialEq)]
enum Operator {
    Fast(),      // *
    Target(),    // :
    Degrade(),   // ?
    Replicate(), // !
}
// TODO: Weight(), // @
// TODO: Slow(),   // /

impl Operator {
    fn parse(pair: Pair<Rule>) -> Result<Operator, String> {
        match pair.as_rule() {
            Rule::op_fast => Ok(Operator::Fast()),
            Rule::op_target => Ok(Operator::Target()),
            Rule::op_degrade => Ok(Operator::Degrade()),
            Rule::op_replicate => Ok(Operator::Replicate()),
            _ => Err(format!("unsupported operator: {:?}", pair.as_rule())),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Expression {
    operator: Operator,
    right: Box<Step>,
    left: Box<Step>,
}

#[derive(Clone, Debug, PartialEq)]
struct Bjorklund {
    left: Box<Step>,
    steps: Box<Step>,
    pulses: Box<Step>,
    rotation: Option<Box<Step>>,
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Pitch {
    note: u8,
    octave: u8,
}

impl Pitch {
    fn parse(pair: Pair<Rule>) -> Pitch {
        let mut pitch = Pitch { note: 0, octave: 4 };
        let mut mark: i8 = 0;
        for p in pair.into_inner() {
            match p.as_rule() {
                Rule::note => {
                    if let Some(c) = String::from(p.as_str()).to_ascii_lowercase().chars().next() {
                        pitch.note = Self::as_note_value(c).unwrap_or(pitch.note)
                    }
                }
                Rule::octave => pitch.octave = p.as_str().parse::<u8>().unwrap_or(pitch.octave),
                Rule::mark => match p.as_str() {
                    "#" => mark = 1,
                    "b" => mark = -1,
                    _ => (),
                },
                _ => (),
            }
        }
        // maybe an error should be thrown instead of a silent clamp
        if pitch.note == 0 && mark == -1 {
            if pitch.octave > 0 {
                pitch.octave -= 1;
                pitch.note = 11;
            }
        } else if pitch.note == 11 && mark == 1 {
            if pitch.octave < 10 {
                pitch.note = 0;
                pitch.octave += 1;
            }
        } else {
            pitch.note = ((pitch.note as i8) + mark) as u8;
        }
        // pitch.note = pitch.note.clamp(0, 127);
        pitch
    }

    pub fn midi_note(&self) -> u8 {
        (self.octave as u32 * 12 + self.note as u32).min(0x7f) as u8
    }

    fn as_note_value(note: char) -> Option<u8> {
        match note {
            'c' => Some(0),
            'd' => Some(2),
            'e' => Some(4),
            'f' => Some(5),
            'g' => Some(7),
            'a' => Some(9),
            'b' => Some(11),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Value {
    #[default]
    Rest,
    Hold,
    Float(f64),
    Integer(i32),
    Pitch(Pitch),
    Name(String),
}

impl Value {
    // parse a single into a value
    // the errors here should be unreachable unless there is a bug in the pest grammar
    fn parse(pair: Pair<Rule>) -> Result<Value, String> {
        // println!("{:?}", pair);
        match pair.as_rule() {
            Rule::number => {
                if let Some(n) = pair.into_inner().next() {
                    match n.as_rule() {
                        Rule::integer => Ok(Value::Integer(n.as_str().parse::<i32>().unwrap_or(0))),
                        Rule::float => Ok(Value::Float(n.as_str().parse::<f64>().unwrap_or(0.0))),
                        Rule::normal => Ok(Value::Float(n.as_str().parse::<f64>().unwrap_or(0.0))),
                        _ => Err(format!("unrecognized number\n{:?}", n)),
                    }
                } else {
                    Err("empty single".to_string())
                }
            }
            Rule::hold => Ok(Value::Hold),
            Rule::rest => Ok(Value::Rest),
            Rule::pitch => Ok(Value::Pitch(Pitch::parse(pair))),
            Rule::name => Ok(Value::Name(pair.as_str().to_string())),
            _ => Err(format!("unrecognized pair in single\n{:?}", pair)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Span {
    start: Fraction,
    end: Fraction,
}

impl Span {
    fn new(start: Fraction, end: Fraction) -> Self {
        Self { start, end }
    }

    // transforms a nested relative span according to an absolute span at output time
    fn transform(&self, outer: &Span) -> Span {
        let start = outer.start + outer.length() * self.start;
        Span {
            start,
            end: start + outer.length() * self.length(),
        }
    }

    pub fn start(&self) -> Fraction {
        self.start
    }

    pub fn end(&self) -> Fraction {
        self.end
    }

    pub fn length(&self) -> Fraction {
        self.end - self.start
    }
}

impl Default for Span {
    fn default() -> Self {
        Span {
            start: Fraction::zero(),
            end: Fraction::one(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Target {
    #[default]
    None,
    Index(i32),
    Name(String),
}

impl Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Target::None => f.write_fmt(format_args!("")),
            _ => {
                f.write_fmt(format_args!(
                    "{:?}",
                    self // self.span.start, self.span.end, self.value
                ))
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Event {
    length: Fraction,
    span: Span,
    value: Value,
    target: Target, // value for instruments
}
impl Event {
    #[cfg(test)]
    fn at(start: Fraction, length: Fraction) -> Self {
        Self {
            length,
            span: Span {
                start,
                end: start + length,
            },
            value: Value::Rest,
            target: Target::None,
        }
    }

    #[cfg(test)]
    fn with_note(&self, note: u8, octave: u8) -> Self {
        Self {
            value: Value::Pitch(Pitch { note, octave }),
            ..self.clone()
        }
    }

    #[cfg(test)]
    fn with_int(&self, i: i32) -> Self {
        Self {
            value: Value::Integer(i),
            ..self.clone()
        }
    }

    #[cfg(test)]
    fn with_name(&self, n: &str) -> Self {
        Self {
            value: Value::Name(n.to_string()),
            ..self.clone()
        }
    }

    #[cfg(test)]
    fn with_hold(&self) -> Self {
        Self {
            value: Value::Hold,
            ..self.clone()
        }
    }

    #[cfg(test)]
    fn with_float(&self, f: f64) -> Self {
        Self {
            value: Value::Float(f),
            ..self.clone()
        }
    }

    #[cfg(test)]
    fn with_target(&self, target: Target) -> Self {
        Self {
            target,
            ..self.clone()
        }
    }

    fn extend(&mut self, next: &Event) {
        self.length += next.length;
        self.span.end = next.span.end
    }

    pub fn value(&self) -> Value {
        self.value.clone()
    }

    pub fn span(&self) -> Span {
        self.span.clone()
    }

    pub fn length(&self) -> Fraction {
        self.length
    }

    pub fn target(&self) -> Target {
        self.target.clone()
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{:.3} -> {:.3} | {:?} {}",
            self.span.start,
            self.span.end,
            self.value,
            self.target // self.span.start, self.span.end, self.value
        ))
    }
}

#[derive(Debug, Clone)]
struct MultiEvents {
    length: Fraction,
    span: Span,
    events: Vec<Events>,
}

#[derive(Debug, Clone)]
struct PolyEvents {
    length: Fraction,
    span: Span,
    channels: Vec<Events>,
}

#[derive(Debug, Clone)]
enum Events {
    Single(Event),
    Multi(MultiEvents),
    Poly(PolyEvents),
}

impl Events {
    fn empty() -> Events {
        Events::Single(Event {
            length: Fraction::one(),
            span: Span::default(),
            value: Value::Rest,
            target: Target::None,
        })
    }

    // only applied for Subdivision and Polymeter groups
    fn subdivide_lengths(events: &mut Vec<Events>) {
        let mut length = Fraction::zero();
        for e in &mut *events {
            match e {
                Events::Single(s) => length += s.length,
                Events::Multi(m) => length += m.length,
                Events::Poly(p) => length += p.length,
            }
        }
        let step_size = Fraction::one() / length;
        let mut start = Fraction::zero();
        for e in &mut *events {
            match e {
                Events::Single(s) => {
                    s.length *= step_size;
                    s.span = Span::new(start, start + s.length);
                    start += s.length
                }
                Events::Multi(m) => {
                    m.length *= step_size;
                    m.span = Span::new(start, start + m.length);
                    start += m.length
                }
                Events::Poly(p) => {
                    p.length *= step_size;
                    p.span = Span::new(start, start + p.length);
                    start += p.length
                }
            }
        }
    }

    fn mutate_events<F>(&mut self, fun: &mut F)
    where
        F: FnMut(&mut Event),
    {
        match self {
            Events::Single(s) => {
                fun(s);
            }
            Events::Multi(m) => {
                for e in &mut m.events {
                    e.mutate_events(fun);
                }
            }
            Events::Poly(p) => {
                for e in &mut p.channels {
                    e.mutate_events(fun);
                }
            }
        }
    }

    fn flatten(&self, channels: &mut Vec<Vec<Event>>, channel: usize) {
        if channels.len() <= channel {
            channels.push(vec![])
        }
        match self {
            Events::Single(s) => channels[channel].push(s.clone()),
            Events::Multi(m) => {
                for e in &m.events {
                    e.flatten(channels, channel);
                }
            }
            Events::Poly(p) => {
                let mut c = channel;
                for e in &p.channels {
                    e.flatten(channels, c);
                    c += 1
                }
            }
        }
    }

    // filter out holds while extending preceding events
    fn merge_holds(events: &[Event]) -> Vec<Event> {
        let mut result: Vec<Event> = vec![];
        for e in events {
            match e.value {
                Value::Hold => {
                    if let Some(last) = result.last_mut() {
                        last.extend(e)
                    }
                }
                _ => result.push(e.clone()),
            }
        }
        result
    }

    // filter out consecutive rests
    // so any remaining rest can be converted to a note-off later
    // rests at the beginning of a pattern also get dropped
    fn merge_rests(events: &[Event]) -> Vec<Event> {
        let mut result: Vec<Event> = vec![];
        for e in events {
            match e.value {
                Value::Rest => {
                    if let Some(last) = result.last() {
                        match last.value {
                            Value::Rest => {}
                            _ => result.push(e.clone()),
                        }
                    }
                }
                _ => result.push(e.clone()),
            }
        }
        result
    }

    // merge holds then rests separately to avoid collapsing rests and holding notes before them
    fn merge(&self, channels: &mut [Vec<Event>]) -> Vec<Vec<Event>> {
        channels
            .iter_mut()
            .map(|e| Self::merge_holds(e))
            .map(|e| Self::merge_rests(&e))
            .collect()
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Parser)]
#[grammar = "tidal/cycle.pest"]
struct CycleParser {}

#[derive(Debug, Clone, PartialEq)]
pub struct Cycle {
    root: Step,
    iteration: u32,
    input: String,
    rng: Xoshiro256PlusPlus,
    seed: Option<[u8; 32]>,
}

impl Cycle {
    // stacks can only appear inside groups like Subdivision, Alternating or Polymeter
    // they will have a stack of steps with their parent's type inside
    fn parse_stack(pair: Pair<Rule>, parent: Pair<Rule>) -> Result<Step, String> {
        let mut channels = vec![];

        match parent.as_rule() {
            Rule::subdivision | Rule::mini | Rule::alternating | Rule::polymeter => {
                for p in pair.into_inner() {
                    let section = Cycle::parse_section(p)?;
                    channels.push(section);
                }
            }
            _ => return Err(format!("invalid parent to stack\n{:?}", parent)),
        }

        let mut stack = Stack { stack: vec![] };

        match parent.as_rule() {
            Rule::alternating => {
                for c in channels {
                    stack.stack.push(Step::Alternating(Alternating {
                        current: 0,
                        steps: c,
                    }))
                }
            }
            Rule::subdivision | Rule::mini => {
                for c in channels {
                    stack
                        .stack
                        .push(Step::Subdivision(Subdivision { steps: c }))
                }
            }
            Rule::polymeter => {
                let count = Cycle::parse_polymeter_count(&parent)?;
                for c in channels {
                    stack.stack.push(Step::Polymeter(Polymeter {
                        offset: 0,
                        steps: c,
                        count,
                    }))
                }
            }
            _ => return Err(format!("invalid parent to stack\n{:?}", parent)),
        }
        Ok(Step::Stack(stack))
    }

    fn parse_polymeter_count(pair: &Pair<Rule>) -> Result<usize, String> {
        for p in pair.clone().into_inner() {
            if p.as_rule() == Rule::polymeter_tail {
                if let Some(count) = p.into_inner().next() {
                    return Ok(count.as_str().parse::<usize>().unwrap_or(1));
                }
            }
            // TODO allow more generic parameter here
        }
        Err(format!("invalid polymeter count\n{:?}", pair))
    }

    fn parse_polymeter(pair: Pair<Rule>) -> Result<Step, String> {
        let count = Cycle::parse_polymeter_count(&pair)?;
        let mut inner = pair.clone().into_inner();
        if let Some(poly_list) = inner.next() {
            return Ok(Step::Polymeter(Polymeter {
                count,
                offset: 0,
                steps: Cycle::parse_section(poly_list).unwrap_or_default(),
            }));
        }
        Err(format!("invalid polymeter\n{:?}", pair))
    }

    // helper to convert a section rule to a vector of Steps
    fn parse_section(pair: Pair<Rule>) -> Result<Vec<Step>, String> {
        let mut steps: Vec<Step> = vec![];
        for pair in pair.into_inner() {
            match Cycle::parse_step(pair) {
                Ok(s) => match s {
                    Step::Repeat => {
                        if let Some(last) = steps.last() {
                            steps.push(last.clone())
                        }else{
                            steps.push(Step::Single(Single::default()))
                            // return Err(String::from("repeat must have a preceding value"))
                        }
                    }
                    _ => steps.push(s)
                }
                Err(s) => return Err(format!("failed to parse section\n{:?}", s)),
            }
        }
        Ok(steps)
    }

    // helper to convert a section or single to a vector of Steps
    fn extract_section(pair: Pair<Rule>) -> Result<Vec<Step>, String> {
        if let Some(inner) = pair.into_inner().next() {
            match inner.as_rule() {
                Rule::repeat => Ok(vec![Step::Repeat]),
                Rule::single => {
                    let single = Step::parse_single(inner)?;
                    Ok(vec![single])
                }
                Rule::section => Cycle::parse_section(inner),
                Rule::choices => {
                    let mut choices: Vec<Step> = vec![];
                    for p in inner.clone().into_inner() {
                        if let Some(step) = p.into_inner().next() {
                            let choice = Cycle::parse_step(step)?;
                            choices.push(choice)
                        } else {
                            return Err(format!("empty choice\n{:?}", inner));
                        }
                    }
                    Ok(vec![Step::Choices(Choices { choices })])
                }
                _ => Err(format!("unexpected rule in section\n{:?}", inner)),
            }
        } else {
            Err("empty section".to_string())
        }
    }

    // recursively parse a pair as a Step
    // errors here should be unreachable unless there is a bug in the pest grammar
    fn parse_step(pair: Pair<Rule>) -> Result<Step, String> {
        match pair.as_rule() {
            Rule::repeat => Ok(Step::Repeat),
            Rule::single => Step::parse_single(pair),
            Rule::subdivision | Rule::mini => {
                if let Some(first) = pair.clone().into_inner().next() {
                    match first.as_rule() {
                        Rule::stack => Cycle::parse_stack(first, pair),
                        _ => {
                            let sd = Subdivision {
                                steps: Cycle::extract_section(pair).unwrap_or_default(),
                            };
                            Ok(Step::Subdivision(sd))
                        }
                    }
                } else {
                    Ok(Step::Single(Single::default()))
                }
            }
            Rule::alternating => {
                if let Some(first) = pair.clone().into_inner().next() {
                    match first.as_rule() {
                        Rule::stack => Cycle::parse_stack(first, pair),
                        _ => {
                            let a = Alternating {
                                current: 0,
                                steps: Cycle::extract_section(pair).unwrap_or_default(),
                            };
                            Ok(Step::Alternating(a))
                        }
                    }
                } else {
                    Ok(Step::Single(Single::default()))
                }
            }
            Rule::polymeter => {
                if let Some(first) = pair.clone().into_inner().next() {
                    match first.as_rule() {
                        Rule::stack => Cycle::parse_stack(first, pair),
                        Rule::polymeter_tail => Ok(Step::Single(Single::default())),
                        _ => Cycle::parse_polymeter(pair),
                    }
                } else {
                    Ok(Step::Single(Single::default()))
                }
            }
            Rule::stack | Rule::section | Rule::choices => {
                // stacks can only appear inside rules for Subdivision, Alternating or Polymeter
                // sections and choices are always immediately handled within other rules
                // using Cycle::extract_section or Cycle::parse_section
                Err(format!("unexpected pair\n{:?}", pair))
            }
            Rule::expr => {
                let mut inner = pair.clone().into_inner();
                match inner.next() {
                    None => Err(format!("empty expression\n{:?}", pair)),
                    Some(left_pair) => {
                        let left = Cycle::parse_step(left_pair)?;
                        match inner.next() {
                            None => Err(format!("incomplete expression\n{:?}", pair)),
                            Some(op) => match op.as_rule() {
                                Rule::op_bjorklund => {
                                    let mut op_inner = op.into_inner();
                                    if let Some(pulse_pair) = op_inner.next() {
                                        let pulses = Cycle::parse_step(pulse_pair)?;
                                        if let Some(steps_pair) = op_inner.next() {
                                            let steps = Cycle::parse_step(steps_pair)?;
                                            let mut rotate = None;
                                            if let Some(rotate_pair) = op_inner.next() {
                                                rotate = Some(Cycle::parse_step(rotate_pair)?);
                                            }
                                            return Ok(Step::Bjorklund(Bjorklund {
                                                left: Box::new(left),
                                                pulses: Box::new(pulses),
                                                steps: Box::new(steps),
                                                rotation: rotate.map(Box::new),
                                            }));
                                        }
                                    }
                                    Err(format!("invalid bjorklund\n{:?}", pair))
                                }
                                _ => {
                                    let operator = Operator::parse(op.clone())?;
                                    let mut inner = op.into_inner();
                                    match inner.next() {
                                        None => match operator {
                                            Operator::Degrade() => {
                                                Ok(Step::Expression(Expression {
                                                    left: Box::new(left),
                                                    right: Box::new(Step::Single(
                                                        Single {
                                                            value: Value::Float(0.5),
                                                            string: "0.5".to_string()
                                                        }
                                                    )),
                                                    operator
                                                }))
                                            }
                                            _ => {
                                                Err(format!(
                                                    "missing right hand side in expression\n{:?}",
                                                    inner
                                                ))
                                            }
                                        }
                                        Some(right_pair) => {
                                            let right = Cycle::parse_step(right_pair)?;
                                            match right {
                                                    Step::Single(_) =>{
                                                        let expr = Step::Expression(Expression {
                                                            left: Box::new(left),
                                                            right: Box::new(right),
                                                            operator,
                                                        });
                                                        Ok(expr)
                                                    }
                                                    _ => Err("only single values are supported on the right hand side".to_string())
                                                }
                                        }
                                    }
                                }
                            },
                        }
                    }
                }
            }
            _ => Err(format!("rule not implemented\n{:?}", pair)),
        }
    }

    // recursively output events for the entire cycle based on some state (random seed)
    fn output(step: &mut Step, rng: &mut Xoshiro256PlusPlus) -> Events {
        match step {
            // repeats only make it here if they had no preceding value
            Step::Repeat => Events::empty(), 
            Step::Single(s) => Events::Single(Event {
                length: Fraction::one(),
                target: Target::None,
                span: Span::default(),
                value: s.value.clone(),
            }),
            Step::Subdivision(sd) => {
                if sd.steps.is_empty() {
                    Events::empty()
                } else {
                    let mut events = vec![];
                    for s in &mut sd.steps {
                        let e = Cycle::output(s, rng);
                        events.push(e)
                        // events.extend(output_events(s, rng))
                    }
                    // only applied for Subdivision and Polymeter groups
                    Events::subdivide_lengths(&mut events);
                    Events::Multi(MultiEvents {
                        span: Span::default(),
                        length: Fraction::one(),
                        events,
                    })
                }
            }
            Step::Alternating(a) => {
                if a.steps.is_empty() {
                    Events::empty()
                } else {
                    let current = a.current % a.steps.len();
                    a.current += 1;
                    if let Some(step) = a.steps.get_mut(current) {
                        Cycle::output(step, rng)
                    } else {
                        Events::empty() // this can never happen
                    }
                }
            }
            Step::Choices(cs) => {
                let choice = rng.gen_range(0..cs.choices.len());
                Cycle::output(&mut cs.choices[choice], rng)
            }
            Step::Polymeter(pm) => {
                if pm.steps.is_empty() {
                    Events::empty()
                } else {
                    let mut events = vec![];
                    let length = pm.steps.len();
                    let offset = pm.offset;

                    for i in 0..pm.count {
                        events.push(Cycle::output(&mut pm.steps[(offset + i) % length], rng))
                    }
                    pm.offset += pm.count;
                    // only applied for Subdivision and Polymeter groups
                    Events::subdivide_lengths(&mut events);
                    Events::Multi(MultiEvents {
                        span: Span::default(),
                        length: Fraction::one(),
                        events,
                    })
                }
            }
            Step::Stack(st) => {
                if st.stack.is_empty() {
                    Events::empty()
                } else {
                    let mut channels = vec![];
                    for s in &mut st.stack {
                        channels.push(Cycle::output(s, rng))
                    }
                    Events::Poly(PolyEvents {
                        span: Span::default(),
                        length: Fraction::one(),
                        channels,
                    })
                }
            }
            Step::Expression(e) => {
                match e.operator {
                    Operator::Fast() => {
                        let mut events = vec![];
                        #[allow(clippy::single_match)]
                        // TODO support something other than Step::Single as the right hand side
                        match e.right.as_ref() {
                            Step::Single(s) => {
                                if let Some(mult) = s.to_integer() {
                                    for _i in 0..mult {
                                        events.push(Cycle::output(&mut e.left, rng))
                                    }
                                }
                            }
                            _ => (),
                        }
                        Events::subdivide_lengths(&mut events);
                        Events::Multi(MultiEvents {
                            span: Span::default(),
                            length: Fraction::one(),
                            events,
                        })
                    }
                    Operator::Target() => {
                        let mut out = Cycle::output(e.left.as_mut(), rng);
                        #[allow(clippy::single_match)]
                        // TODO support something other than Step::Single as the right hand side
                        match e.right.as_ref() {
                            Step::Single(s) => out.mutate_events(&mut |e| e.target = s.to_target()),
                            _ => (),
                        }
                        out
                    }
                    Operator::Degrade() => {
                        let mut out = Cycle::output(e.left.as_mut(), rng);
                        #[allow(clippy::single_match)]
                        // TODO support something other than Step::Single as the right hand side
                        match e.right.as_ref() {
                            Step::Single(s) => out.mutate_events(&mut |e: &mut Event| {
                                if let Some(chance) = s.to_chance() {
                                    if chance < rng.gen_range(0.0..1.0) {
                                        e.value = Value::Rest
                                    }
                                }
                            }),
                            _ => (),
                        }
                        out
                    }
                    Operator::Replicate() => {
                        let mut events = vec![];
                        let mut length = Fraction::from(1);
                        #[allow(clippy::single_match)]
                        // TODO support something other than Step::Single as the right hand side
                        match e.right.as_ref() {
                            Step::Single(s) => {
                                if let Some(mult) = s.to_integer() {
                                    length = Fraction::from(mult);
                                    let out = Cycle::output(&mut e.left, rng);
                                    for _i in 0..mult {
                                        events.push(out.clone())
                                    }
                                }
                            }
                            _ => (),
                        }
                        Events::subdivide_lengths(&mut events);
                        Events::Multi(MultiEvents {
                            span: Span::default(),
                            length,
                            events,
                        })
                    }
                }
            }
            Step::Bjorklund(b) => {
                let mut events = vec![];
                #[allow(clippy::single_match)]
                // TODO support something other than Step::Single as the right hand side
                match b.pulses.as_ref() {
                    Step::Single(steps_single) => {
                        match b.steps.as_ref() {
                            Step::Single(pulses_single) => {
                                let rotation = match &b.rotation {
                                    Some(r) => match r.as_ref() {
                                        Step::Single(rotation_single) => {
                                            rotation_single.to_integer()
                                        }
                                        _ => None, // TODO support something other than Step::Single as rotation
                                    },
                                    None => None,
                                };
                                if let Some(steps) = steps_single.to_integer() {
                                    if let Some(pulses) = pulses_single.to_integer() {
                                        let out = Cycle::output(&mut b.left, rng);
                                        for pulse in euclidean(
                                            steps.max(0) as u32,
                                            pulses.max(0) as u32,
                                            rotation.unwrap_or(0),
                                        ) {
                                            if pulse {
                                                events.push(out.clone())
                                            } else {
                                                events.push(Events::empty())
                                            }
                                        }
                                    }
                                }
                            }
                            _ => (), // TODO support something other than Step::Single as steps
                        }
                    }
                    _ => (), // TODO support something other than Step::Single as pulses
                }
                Events::subdivide_lengths(&mut events);
                Events::Multi(MultiEvents {
                    span: Span::default(),
                    length: Fraction::one(),
                    events,
                })
            } // _ => Events::Single(SingleEvent::default())
        }
    }

    // recursively transform the spans of events from relative time to absolute
    fn transform_spans(events: &mut Events, span: &Span) {
        let unit = span.length();
        match events {
            Events::Single(s) => {
                s.length *= unit;
                s.span = s.span.transform(span);
            }
            Events::Multi(m) => {
                m.length *= unit;
                m.span = m.span.transform(span);

                for e in &mut m.events {
                    Cycle::transform_spans(e, &m.span);
                }
            }
            Events::Poly(p) => {
                p.length *= unit;
                p.span = p.span.transform(span);
                for e in &mut p.channels {
                    Cycle::transform_spans(e, &p.span);
                }
            }
        }
    }

    // reset state to initial state
    pub fn reset(&mut self) {
        self.iteration = 0;
        self.root.reset();
        self.rng = Xoshiro256PlusPlus::from_seed(self.seed.unwrap_or_else(|| thread_rng().gen()));
    }

    // parse the root pair of the pest AST into a Subdivision
    // then update the spans of all the generated steps
    pub fn generate(&mut self) -> Vec<Vec<Event>> {
        let mut events = Cycle::output(&mut self.root, &mut self.rng);
        Cycle::transform_spans(&mut events, &Span::default());
        let mut channels = vec![];
        events.flatten(&mut channels, 0);
        events.merge(&mut channels);

        #[cfg(test)]
        {
            println!("\nOUTPUT {}", self.iteration);
            let channel_count = channels.len();
            for (ci, channel) in channels.iter().enumerate() {
                if channel_count > 1 {
                    println!(" /{}", ci);
                }
                for (i, event) in channel.iter().enumerate() {
                    println!("  │{:02}│ {}", i, event);
                }
            }
        }

        self.iteration += 1;
        channels
    }

    pub fn is_stateful(&self) -> bool {
        ['<', '{', '|', '?'].iter().any(|&c| self.input.contains(c))
    }

    pub fn from(input: &str, seed: Option<[u8; 32]>) -> Result<Self, String> {
        match CycleParser::parse(Rule::mini, input) {
            Ok(mut tree) => {
                if let Some(mini) = tree.next() {
                    #[cfg(test)]
                    {
                        println!("\nTREE");
                        Self::print_pairs(&mini, 0);
                    }
                    let root = Cycle::parse_step(mini)?;
                    let rng =
                        Xoshiro256PlusPlus::from_seed(seed.unwrap_or_else(|| thread_rng().gen()));
                    let iteration = 0;
                    let input = input.to_string();
                    let cycle = Self {
                        root,
                        rng,
                        iteration,
                        input,
                        seed,
                    };
                    #[cfg(test)]
                    {
                        println!("\nCYCLE");
                        cycle.print();
                    }
                    Ok(cycle)
                } else {
                    Err("couldn't parse input".to_string())
                }
            }
            Err(err) => Err(format!("{}", err)),
        }
    }

    #[cfg(test)]
    fn indent_lines(level: usize) -> String {
        let mut lines = String::new();
        for i in 0..level {
            lines += [" │", " |"][i % 2];
        }
        lines
    }

    #[cfg(test)]
    fn print_steps(step: &Step, level: usize) {
        let name = match step {
            Step::Repeat => "Repeat".to_string(),
            Step::Single(s) => match &s.value {
                Value::Pitch(_p) => format!("{:?} {}", s.value, s.string),
                _ => format!("{:?} {:?}", s.value, s.string),
            },
            Step::Subdivision(sd) => format!("{} [{}]", "Subdivision", sd.steps.len()),
            Step::Alternating(a) => format!("{} <{}>", "Alternating", a.steps.len()),
            Step::Polymeter(pm) => format!("{} {{{}}}", "Polymeter", pm.steps.len()),
            Step::Choices(cs) => format!("{} |{}|", "Choices", cs.choices.len()),
            Step::Stack(st) => format!("{} ({})", "Stack", st.stack.len()),
            Step::Expression(e) => format!("Expression {:?}", e.operator),
            Step::Bjorklund(_b) => format!("Bjorklund {}", ""),
        };
        println!("{} {}", Self::indent_lines(level), name);
        for step in step.inner_steps() {
            Self::print_steps(step, level + 1)
        }
    }

    #[cfg(test)]
    fn print_pairs(pair: &Pair<Rule>, level: usize) {
        println!(
            "{} {:?} {:?}",
            Self::indent_lines(level),
            pair.as_rule(),
            pair.as_str()
        );
        for p in pair.clone().into_inner() {
            Self::print_pairs(&p, level + 1)
        }
    }

    #[cfg(test)]
    fn print(&self) {
        Self::print_steps(&self.root, 0);
    }
}

// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    type F = fraction::Fraction;

    fn assert_cycles(input: &str, outputs: Vec<Vec<Vec<Event>>>) -> Result<(), String> {
        let mut cycle = Cycle::from(input, None)?;
        for out in outputs {
            assert_eq!(cycle.generate(), out);
        }
        Ok(())
    }

    #[test]

    pub fn cycle() -> Result<(), String> {
        assert_eq!(
            Cycle::from("a b c d", None)?.generate(),
            [[
                Event::at(F::from(0), F::new(1u8, 4u8)).with_note(9, 4),
                Event::at(F::new(1u8, 4u8), F::new(1u8, 4u8)).with_note(11, 4),
                Event::at(F::new(2u8, 4u8), F::new(1u8, 4u8)).with_note(0, 4),
                Event::at(F::new(3u8, 4u8), F::new(1u8, 4u8)).with_note(2, 4),
            ]]
        );
        assert_eq!(
            Cycle::from("\ta\r\n\tb\nc\n d\n\n", None)?.generate(),
            Cycle::from("a b c d", None)?.generate()
        );
        assert_eq!(
            Cycle::from("a b [ c d ]", None)?.generate(),
            [[
                Event::at(F::from(0), F::new(1u8, 3u8)).with_note(9, 4),
                Event::at(F::new(1u8, 3u8), F::new(1u8, 3u8)).with_note(11, 4),
                Event::at(F::new(2u8, 3u8), F::new(1u8, 6u8)).with_note(0, 4),
                Event::at(F::new(5u8, 6u8), F::new(1u8, 6u8)).with_note(2, 4),
            ]]
        );
        assert_eq!(
            Cycle::from("[a a] [b4 b5 b6] [c0 d1 c2 d3]", None)?.generate(),
            [[
                Event::at(F::from(0), F::new(1u8, 6u8)).with_note(9, 4),
                Event::at(F::new(1u8, 6u8), F::new(1u8, 6u8)).with_note(9, 4),
                Event::at(F::new(3u8, 9u8), F::new(1u8, 9u8)).with_note(11, 4),
                Event::at(F::new(4u8, 9u8), F::new(1u8, 9u8)).with_note(11, 5),
                Event::at(F::new(5u8, 9u8), F::new(1u8, 9u8)).with_note(11, 6),
                Event::at(F::new(8u8, 12u8), F::new(1u8, 12u8)).with_note(0, 0),
                Event::at(F::new(9u8, 12u8), F::new(1u8, 12u8)).with_note(2, 1),
                Event::at(F::new(10u8, 12u8), F::new(1u8, 12u8)).with_note(0, 2),
                Event::at(F::new(11u8, 12u8), F::new(1u8, 12u8)).with_note(2, 3),
            ]]
        );
        assert_eq!(
            Cycle::from("[a0 [bb1 [b2 c3]]] c#4 [[[d5 D#6] E7 ] F8]", None)?.generate(),
            [[
                Event::at(F::from(0), F::new(1u8, 6u8)).with_note(9, 0),
                Event::at(F::new(1u8, 6u8), F::new(1u8, 12u8)).with_note(10, 1),
                Event::at(F::new(3u8, 12u8), F::new(1u8, 24u8)).with_note(11, 2),
                Event::at(F::new(7u8, 24u8), F::new(1u8, 24u8)).with_note(0, 3),
                Event::at(F::new(1u8, 3u8), F::new(1u8, 3u8)).with_note(1, 4),
                Event::at(F::new(2u8, 3u8), F::new(1u8, 24u8)).with_note(2, 5),
                Event::at(F::new(17u8, 24u8), F::new(1u8, 24u8)).with_note(3, 6),
                Event::at(F::new(9u8, 12u8), F::new(1u8, 12u8)).with_note(4, 7),
                Event::at(F::new(5u8, 6u8), F::new(1u8, 6u8)).with_note(5, 8),
            ]]
        );
        assert_eq!(
            Cycle::from("[R [e [n o]]] , [[[i s] e ] _]", None)?.generate(),
            [
                [
                    Event::at(F::from(0), F::new(1u8, 2u8)).with_name("R"),
                    Event::at(F::new(1u8, 2u8), F::new(1u8, 4u8)).with_note(4, 4),
                    Event::at(F::new(3u8, 4u8), F::new(1u8, 8u8)).with_name("n"),
                    Event::at(F::new(7u8, 8u8), F::new(1u8, 8u8)).with_name("o"),
                ],
                [
                    Event::at(F::from(0), F::new(1u8, 8u8)).with_name("i"),
                    Event::at(F::new(1u8, 8u8), F::new(1u8, 8u8)).with_name("s"),
                    Event::at(F::new(1u8, 4u8), F::new(1u8, 4u8)).with_note(4, 4),
                    Event::at(F::new(1u8, 2u8), F::new(1u8, 2u8)).with_hold(),
                ],
            ]
        );

        assert_cycles(
            "<a b c d>",
            vec![
                vec![vec![Event::at(F::from(0), F::from(1)).with_note(9, 4)]],
                vec![vec![Event::at(F::from(0), F::from(1)).with_note(11, 4)]],
                vec![vec![Event::at(F::from(0), F::from(1)).with_note(0, 4)]],
                vec![vec![Event::at(F::from(0), F::from(1)).with_note(2, 4)]],
            ],
        )?;

        assert_cycles(
            "<a ~ ~ a0> <b <c d>>",
            vec![
                vec![vec![
                    Event::at(F::from(0), F::new(1u8, 2u8)).with_note(9, 4),
                    Event::at(F::new(1u8, 2u8), F::new(1u8, 2u8)).with_note(11, 4),
                ]],
                vec![vec![
                    Event::at(F::from(0), F::new(1u8, 2u8)),
                    Event::at(F::new(1u8, 2u8), F::new(1u8, 2u8)).with_note(0, 4),
                ]],
                vec![vec![
                    Event::at(F::from(0), F::new(1u8, 2u8)),
                    Event::at(F::new(1u8, 2u8), F::new(1u8, 2u8)).with_note(11, 4),
                ]],
                vec![vec![
                    Event::at(F::from(0), F::new(1u8, 2u8)).with_note(9, 0),
                    Event::at(F::new(1u8, 2u8), F::new(1u8, 2u8)).with_note(2, 4),
                ]],
            ],
        )?;

        assert_cycles(
            "<<a a8> b,  <c [d e]>>",
            vec![
                vec![
                    vec![Event::at(F::from(0), F::from(1)).with_note(9, 4)],
                    vec![Event::at(F::from(0), F::from(1)).with_note(0, 4)],
                ],
                vec![
                    vec![Event::at(F::from(0), F::from(1)).with_note(11, 4)],
                    vec![
                        Event::at(F::from(0), F::new(1u8, 2u8)).with_note(2, 4),
                        Event::at(F::new(1u8, 2u8), F::new(1u8, 2u8)).with_note(4, 4),
                    ],
                ],
                vec![
                    vec![Event::at(F::from(0), F::from(1)).with_note(9, 8)],
                    vec![Event::at(F::from(0), F::from(1)).with_note(0, 4)],
                ],
            ],
        )?;

        assert_cycles(
            "{-3 -2 -1 0 1 2 3}%4",
            vec![
                vec![vec![
                    Event::at(F::from(0), F::new(1u8, 4u8)).with_int(-3),
                    Event::at(F::new(1u8, 4u8), F::new(1u8, 4u8)).with_int(-2),
                    Event::at(F::new(2u8, 4u8), F::new(1u8, 4u8)).with_int(-1),
                    Event::at(F::new(3u8, 4u8), F::new(1u8, 4u8)).with_int(0),
                ]],
                vec![vec![
                    Event::at(F::from(0), F::new(1u8, 4u8)).with_int(1),
                    Event::at(F::new(1u8, 4u8), F::new(1u8, 4u8)).with_int(2),
                    Event::at(F::new(2u8, 4u8), F::new(1u8, 4u8)).with_int(3),
                    Event::at(F::new(3u8, 4u8), F::new(1u8, 4u8)).with_int(-3),
                ]],
                vec![vec![
                    Event::at(F::from(0), F::new(1u8, 4u8)).with_int(-2),
                    Event::at(F::new(1u8, 4u8), F::new(1u8, 4u8)).with_int(-1),
                    Event::at(F::new(2u8, 4u8), F::new(1u8, 4u8)).with_int(0),
                    Event::at(F::new(3u8, 4u8), F::new(1u8, 4u8)).with_int(1),
                ]],
            ],
        )?;

        assert_cycles(
            "{<0 0 d#8:test> 1 <c d e>:3 [<.5 0.95> 1.]}%3",
            vec![
                vec![vec![
                    Event::at(F::from(0), F::new(1u8, 3u8)).with_int(0),
                    Event::at(F::new(1u8, 3u8), F::new(1u8, 3u8)).with_int(1),
                    Event::at(F::new(2u8, 3u8), F::new(1u8, 3u8))
                        .with_note(0, 4)
                        .with_target(Target::Index(3)),
                ]],
                vec![vec![
                    Event::at(F::from(0), F::new(1u8, 6u8)).with_float(0.5),
                    Event::at(F::new(1u8, 6u8), F::new(1u8, 6u8)).with_float(1.0),
                    Event::at(F::new(1u8, 3u8), F::new(1u8, 3u8)).with_int(0),
                    Event::at(F::new(2u8, 3u8), F::new(1u8, 3u8)).with_int(1),
                ]],
                vec![vec![
                    Event::at(F::from(0), F::new(1u8, 3u8))
                        .with_note(2, 4)
                        .with_target(Target::Index(3)),
                    Event::at(F::new(2u8, 6u8), F::new(1u8, 6u8)).with_float(0.95),
                    Event::at(F::new(3u8, 6u8), F::new(1u8, 6u8)).with_float(1.0),
                    Event::at(F::new(2u8, 3u8), F::new(1u8, 3u8))
                        .with_note(3, 8)
                        .with_target(Target::Name("test".to_string())),
                ]],
            ],
        )?;

        assert_eq!(
            Cycle::from("[1 middle _] {}%42 [] <>", None)?.generate(),
            [[
                Event::at(F::from(0), F::new(1u8, 12u8)).with_int(1),
                Event::at(F::new(1u8, 12u8), F::new(1u8, 12u8)).with_name("middle"),
                Event::at(F::new(2u8, 12u8), F::new(1u8, 12u8)).with_hold(),
                Event::at(F::new(1u8, 4u8), F::new(1u8, 4u8)),
                Event::at(F::new(2u8, 4u8), F::new(1u8, 4u8)),
                Event::at(F::new(3u8, 4u8), F::new(1u8, 4u8)),
            ]]
        );

        assert_cycles(
            "[1 2] [3 4,[5 6]:42]",
            vec![vec![
                vec![
                    Event::at(F::from(0), F::new(1u8, 4u8)).with_int(1),
                    Event::at(F::new(1u8, 4u8), F::new(1u8, 4u8)).with_int(2),
                    Event::at(F::new(2u8, 4u8), F::new(1u8, 4u8)).with_int(3),
                    Event::at(F::new(3u8, 4u8), F::new(1u8, 4u8)).with_int(4),
                ],
                vec![
                    Event::at(F::new(1u8, 2u8), F::new(1u8, 4u8))
                        .with_int(5)
                        .with_target(Target::Index(42)),
                    Event::at(F::new(3u8, 4u8), F::new(1u8, 4u8))
                        .with_int(6)
                        .with_target(Target::Index(42)),
                ],
            ]],
        )?;

        assert_eq!(
            Cycle::from("1 second*2 eb3*3 [32 32]*4", None)?.generate(),
            [[
                Event::at(F::from(0), F::new(1u8, 4u8)).with_int(1),
                Event::at(F::new(2u8, 8u8), F::new(1u8, 8u8)).with_name("second"),
                Event::at(F::new(3u8, 8u8), F::new(1u8, 8u8)).with_name("second"),
                Event::at(F::new(6u8, 12u8), F::new(1u8, 12u8)).with_note(3, 3),
                Event::at(F::new(7u8, 12u8), F::new(1u8, 12u8)).with_note(3, 3),
                Event::at(F::new(8u8, 12u8), F::new(1u8, 12u8)).with_note(3, 3),
                Event::at(F::new(24u8, 32u8), F::new(1u8, 32u8)).with_int(32),
                Event::at(F::new(25u8, 32u8), F::new(1u8, 32u8)).with_int(32),
                Event::at(F::new(26u8, 32u8), F::new(1u8, 32u8)).with_int(32),
                Event::at(F::new(27u8, 32u8), F::new(1u8, 32u8)).with_int(32),
                Event::at(F::new(28u8, 32u8), F::new(1u8, 32u8)).with_int(32),
                Event::at(F::new(29u8, 32u8), F::new(1u8, 32u8)).with_int(32),
                Event::at(F::new(30u8, 32u8), F::new(1u8, 32u8)).with_int(32),
                Event::at(F::new(31u8, 32u8), F::new(1u8, 32u8)).with_int(32),
            ]]
        );

        assert_cycles(
            "tresillo(6,8), outside(4,11)",
            vec![vec![
                vec![
                    Event::at(F::from(0), F::new(1u8, 8u8)).with_name("tresillo"),
                    Event::at(F::new(1u8, 8u8), F::new(1u8, 8u8)),
                    Event::at(F::new(2u8, 8u8), F::new(1u8, 8u8)).with_name("tresillo"),
                    Event::at(F::new(3u8, 8u8), F::new(1u8, 8u8)).with_name("tresillo"),
                    Event::at(F::new(4u8, 8u8), F::new(1u8, 8u8)).with_name("tresillo"),
                    Event::at(F::new(5u8, 8u8), F::new(1u8, 8u8)),
                    Event::at(F::new(6u8, 8u8), F::new(1u8, 8u8)).with_name("tresillo"),
                    Event::at(F::new(7u8, 8u8), F::new(1u8, 8u8)).with_name("tresillo"),
                ],
                vec![
                    Event::at(F::from(0), F::new(1u8, 11u8)).with_name("outside"),
                    Event::at(F::new(1u8, 11u8), F::new(1u8, 11u8)),
                    Event::at(F::new(2u8, 11u8), F::new(1u8, 11u8)),
                    Event::at(F::new(3u8, 11u8), F::new(1u8, 11u8)).with_name("outside"),
                    Event::at(F::new(4u8, 11u8), F::new(1u8, 11u8)),
                    Event::at(F::new(5u8, 11u8), F::new(1u8, 11u8)),
                    Event::at(F::new(6u8, 11u8), F::new(1u8, 11u8)).with_name("outside"),
                    Event::at(F::new(7u8, 11u8), F::new(1u8, 11u8)),
                    Event::at(F::new(8u8, 11u8), F::new(1u8, 11u8)),
                    Event::at(F::new(9u8, 11u8), F::new(1u8, 11u8)).with_name("outside"),
                    Event::at(F::new(10u8, 11u8), F::new(1u8, 11u8)),
                ],
            ]],
        )?;

        assert_cycles(
            "[<1 10> <2 20>:a](2,5)",
            vec![
                vec![vec![
                    Event::at(F::from(0), F::new(1u8, 10u8)).with_int(1),
                    Event::at(F::new(1u8, 10u8), F::new(1u8, 10u8))
                        .with_int(2)
                        .with_target(Target::Name("a".to_string())),
                    Event::at(F::new(1u8, 5u8), F::new(1u8, 5u8)),
                    Event::at(F::new(2u8, 5u8), F::new(1u8, 10u8)).with_int(1),
                    Event::at(F::new(5u8, 10u8), F::new(1u8, 10u8))
                        .with_int(2)
                        .with_target(Target::Name("a".to_string())),
                    Event::at(F::new(3u8, 5u8), F::new(1u8, 5u8)),
                    Event::at(F::new(4u8, 5u8), F::new(1u8, 5u8)),
                ]],
                vec![vec![
                    Event::at(F::from(0), F::new(1u8, 10u8)).with_int(10),
                    Event::at(F::new(1u8, 10u8), F::new(1u8, 10u8))
                        .with_int(20)
                        .with_target(Target::Name("a".to_string())),
                    Event::at(F::new(1u8, 5u8), F::new(1u8, 5u8)),
                    Event::at(F::new(2u8, 5u8), F::new(1u8, 10u8)).with_int(10),
                    Event::at(F::new(5u8, 10u8), F::new(1u8, 10u8))
                        .with_int(20)
                        .with_target(Target::Name("a".to_string())),
                    Event::at(F::new(3u8, 5u8), F::new(1u8, 5u8)),
                    Event::at(F::new(4u8, 5u8), F::new(1u8, 5u8)),
                ]],
            ],
        )?;

        assert_eq!(
            Cycle::from("1!2 3 [4!3 5]", None)?.generate(),
            [[
                Event::at(F::from(0), F::new(1u8, 4u8)).with_int(1),
                Event::at(F::new(1u8, 4u8), F::new(1u8, 4u8)).with_int(1),
                Event::at(F::new(2u8, 4u8), F::new(1u8, 4u8)).with_int(3),
                Event::at(F::new(12u8, 16u8), F::new(1u8, 16u8)).with_int(4),
                Event::at(F::new(13u8, 16u8), F::new(1u8, 16u8)).with_int(4),
                Event::at(F::new(14u8, 16u8), F::new(1u8, 16u8)).with_int(4),
                Event::at(F::new(15u8, 16u8), F::new(1u8, 16u8)).with_int(5),
            ]]
        );

        assert_cycles(
            "[0 1]!2 <a b>!2",
            vec![
                vec![vec![
                    Event::at(F::from(0), F::new(1u8, 8u8)).with_int(0),
                    Event::at(F::new(1u8, 8u8), F::new(1u8, 8u8)).with_int(1),
                    Event::at(F::new(2u8, 8u8), F::new(1u8, 8u8)).with_int(0),
                    Event::at(F::new(3u8, 8u8), F::new(1u8, 8u8)).with_int(1),
                    Event::at(F::new(2u8, 4u8), F::new(1u8, 4u8)).with_note(9, 4),
                    Event::at(F::new(3u8, 4u8), F::new(1u8, 4u8)).with_note(9, 4),
                ]],
                vec![vec![
                    Event::at(F::from(0), F::new(1u8, 8u8)).with_int(0),
                    Event::at(F::new(1u8, 8u8), F::new(1u8, 8u8)).with_int(1),
                    Event::at(F::new(2u8, 8u8), F::new(1u8, 8u8)).with_int(0),
                    Event::at(F::new(3u8, 8u8), F::new(1u8, 8u8)).with_int(1),
                    Event::at(F::new(2u8, 4u8), F::new(1u8, 4u8)).with_note(11, 4),
                    Event::at(F::new(3u8, 4u8), F::new(1u8, 4u8)).with_note(11, 4),
                ]],
            ],
        )?;

        assert_eq!(
            Cycle::from("c(3,8,9)", None)?.generate(),
            [[
                Event::at(F::from(0), F::new(1u8, 8u8)),
                Event::at(F::new(1u8, 8u8), F::new(1u8, 8u8)),
                Event::at(F::new(2u8, 8u8), F::new(1u8, 8u8)).with_note(0, 4),
                Event::at(F::new(3u8, 8u8), F::new(1u8, 8u8)),
                Event::at(F::new(4u8, 8u8), F::new(1u8, 8u8)),
                Event::at(F::new(5u8, 8u8), F::new(1u8, 8u8)).with_note(0, 4),
                Event::at(F::new(6u8, 8u8), F::new(1u8, 8u8)),
                Event::at(F::new(7u8, 8u8), F::new(1u8, 8u8)).with_note(0, 4),
            ]]
        );

        assert_eq!(
            Cycle::from("[a b c](3,8,9)", None)?.generate(),
            Cycle::from("[a b c](3,8,1)", None)?.generate()
        );

        assert_eq!(
            Cycle::from("[a b c](3,8,7)", None)?.generate(),
            Cycle::from("[a b c](3,8,-1)", None)?.generate()
        );


        assert_eq!(
            Cycle::from("[a a a a]", None)?.generate(),
            Cycle::from("[a ! ! !]", None)?.generate()
        );

        assert_eq!(
            Cycle::from("[! ! a !]", None)?.generate(),
            Cycle::from("[~ ~ a a]", None)?.generate()
        );


        assert_eq!(
            Cycle::from("a ~ ~ ~", None)?.generate(),
            Cycle::from("a - - -", None)?.generate()
        );

        assert_eq!(
            Cycle::from("a? b?", None)?.root,
            Cycle::from("a?0.5 b?0.5", None)?.root
        );

        assert_eq!(
            Cycle::from("[a b] ! ! <a b c> !", None)?.generate(),
            Cycle::from("[a b] [a b] [a b] <a b c> <a b c>", None)?.generate()
        );

        // TODO test random outputs // parse_with_debug("[a b c d]?0.5");

        assert!(Cycle::from("a b c [d", None).is_err());
        assert!(Cycle::from("a b/ c [d", None).is_err());
        assert!(Cycle::from("a b--- c [d", None).is_err());
        assert!(Cycle::from("*a b c [d", None).is_err());
        assert!(Cycle::from("a {{{}}", None).is_err());
        assert!(Cycle::from("a*[]", None).is_err());
        assert!(Cycle::from("] a z [", None).is_err());
        assert!(Cycle::from("->err", None).is_err());
        assert!(Cycle::from("(a, b)", None).is_err());
        assert!(Cycle::from("#(12, 32)", None).is_err());
        assert!(Cycle::from("#c $", None).is_err());
        assert!(Cycle::from("1.. 2 3", None).is_err());
        assert!(Cycle::from("1 ..2 3", None).is_err());

        Ok(())
    }
}
