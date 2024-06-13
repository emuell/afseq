#[cfg(test)]
use core::fmt::Display;

use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

use rand::{thread_rng, Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use fraction::ToPrimitive;
use fraction::{Fraction, One, Zero};

use crate::pattern::euclidean::euclidean;

// -------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct Cycle {
    root: Step,
    event_limit: usize,
    input: String,
    seed: Option<[u8; 32]>,
    state: CycleState,
}
impl Cycle {
    /// create a Cycle from an mini-notation string with an optional seed
    pub fn from(input: &str, seed: Option<[u8; 32]>) -> Result<Self, String> {
        match CycleParser::parse(Rule::mini, input) {
            Ok(mut tree) => {
                if let Some(mini) = tree.next() {
                    #[cfg(test)]
                    {
                        println!("\nTREE");
                        Self::print_pairs(&mini, 0);
                    }
                    let input = input.to_string();
                    let root = CycleParser::step(mini)?;
                    let state = CycleState {
                        step: 0,
                        events: 0,
                        iteration: 0,
                        rng: Xoshiro256PlusPlus::from_seed(
                            seed.unwrap_or_else(|| thread_rng().gen()),
                        ),
                    };
                    let cycle = Self {
                        input,
                        seed,
                        root,
                        state,
                        event_limit: 0x1000,
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

    // TODO remove this or improve, * and / can change the output, <1> does not etc..
    /// check if a cycle will give different outputs between cycles
    pub fn is_stateful(&self) -> bool {
        ['<', '{', '|', '?', '/']
            .iter()
            .any(|&c| self.input.contains(c))
    }

    /// query for the next iteration of output
    pub fn generate(&mut self) -> Result<Vec<Vec<Event>>, String> {
        let cycle = self.state.iteration;
        self.state.events = 0;
        self.state.step = 0;
        let mut events = Self::output(&self.root, &mut self.state, cycle, self.event_limit)?;
        self.state.iteration += 1;
        events.transform_spans(&Span::default());
        Ok(events.export())
    }

    /// reset state to initial state
    pub fn reset(&mut self) {
        self.state.iteration = 0;
        self.state.step = 0;
        self.state.events = 0;
        self.state.rng =
            Xoshiro256PlusPlus::from_seed(self.seed.unwrap_or_else(|| thread_rng().gen()));
    }
}

#[derive(Debug, Clone, Default)]
pub struct Event {
    length: Fraction,
    span: Span,
    value: Value,
    string: String,
    target: Target,
}

impl Event {
    /// The step's original parsed value.
    pub fn value(&self) -> &Value {
        &self.value
    }

    /// The step's original value string, unparsed.
    pub fn string(&self) -> &str {
        &self.string
    }

    /// The step's time span.
    pub fn span(&self) -> &Span {
        &self.span
    }

    /// The step's length.
    pub fn length(&self) -> &Fraction {
        &self.length
    }

    /// The step's optional value target.
    pub fn target(&self) -> &Target {
        &self.target
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Span {
    start: Fraction,
    end: Fraction,
}

impl Span {
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

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Target {
    #[default]
    None,
    Index(i32),
    Name(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Pitch {
    note: u8,
    octave: u8,
}

impl Pitch {
    pub fn midi_note(&self) -> u8 {
        (self.octave as u32 * 12 + self.note as u32).min(0x7f) as u8
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
enum Step {
    Single(Single),
    Alternating(Alternating),
    Subdivision(Subdivision),
    Polymeter(Polymeter),
    Stack(Stack),
    Choices(Choices),
    DynamicExpression(DynamicExpression),
    StaticExpression(StaticExpression),
    Bjorklund(Bjorklund),
    Range(Range),
    Repeat,
}

impl Step {
    #[allow(dead_code)]
    fn inner_steps(&self) -> Vec<&Step> {
        match self {
            Step::Repeat => vec![],
            Step::Single(_s) => vec![],
            Step::Alternating(a) => a.steps.iter().collect(),
            Step::Polymeter(pm) => pm.steps.as_ref().inner_steps(),
            Step::Subdivision(sd) => sd.steps.iter().collect(),
            Step::Choices(cs) => cs.choices.iter().collect(),
            Step::Stack(st) => st.stack.iter().collect(),
            Step::DynamicExpression(e) => vec![&e.left, &e.right],
            Step::StaticExpression(e) => vec![&e.left],
            Step::Range(_) => vec![],
            Step::Bjorklund(b) => {
                if let Some(rotation) = &b.rotation {
                    vec![&b.left, &b.steps, &b.pulses, &**rotation]
                } else {
                    vec![&b.left, &b.steps, &b.pulses]
                }
            }
        }
    }
    fn rest() -> Self {
        Self::Single(Single::default())
    }
    fn subdivision(steps: Vec<Step>) -> Self {
        Self::Subdivision(Subdivision { steps })
    }
    fn alternating(steps: Vec<Step>) -> Self {
        Self::Alternating(Alternating { steps })
    }
    fn polymeter(steps: Vec<Step>, count: Step) -> Self {
        Step::Polymeter(Polymeter {
            steps: Box::new(Step::subdivision(steps)),
            count: Box::new(count),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Single {
    value: Value,
    string: String,
}

impl Default for Single {
    fn default() -> Self {
        Single {
            value: Value::Rest,
            string: String::from("~"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Alternating {
    steps: Vec<Step>,
}

#[derive(Clone, Debug, PartialEq)]
struct Subdivision {
    steps: Vec<Step>,
}

#[derive(Clone, Debug, PartialEq)]
struct Polymeter {
    count: Box<Step>,
    steps: Box<Step>,
}

impl Polymeter {
    fn length(&self) -> usize {
        if let Step::Subdivision(s) = self.steps.as_ref() {
            s.steps.len()
        } else {
            1
        } // unreachable
    }
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
enum DynamicOp {
    Fast(),      // *
    Slow(),      // /
    Bjorklund(), // (p,s,r)
}

#[derive(Clone, Debug, PartialEq)]
enum StaticOp {
    Target(),    // :
    Degrade(),   // ?
    Replicate(), // !
    Weight(),    // @
}

impl StaticOp {
    fn default_value(&self) -> Value {
        match self {
            StaticOp::Weight() | StaticOp::Replicate() => Value::Integer(2),
            StaticOp::Degrade() => Value::Float(0.5),
            StaticOp::Target() => Value::Rest,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Operator {
    Static(StaticOp),
    Dynamic(DynamicOp),
}

impl Operator {
    fn parse(pair: Pair<Rule>) -> Result<Self, String> {
        match pair.as_rule() {
            Rule::op_target => Ok(Self::Static(StaticOp::Target())),
            Rule::op_degrade => Ok(Self::Static(StaticOp::Degrade())),
            Rule::op_replicate => Ok(Self::Static(StaticOp::Replicate())),
            Rule::op_weight => Ok(Self::Static(StaticOp::Weight())),
            Rule::op_fast => Ok(Self::Dynamic(DynamicOp::Fast())),
            Rule::op_slow => Ok(Self::Dynamic(DynamicOp::Slow())),
            Rule::op_bjorklund => Ok(Self::Dynamic(DynamicOp::Bjorklund())),
            _ => Err(format!("unsupported operator: {:?}", pair.as_rule())),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct DynamicExpression {
    op: DynamicOp,
    left: Box<Step>,
    right: Box<Step>,
}

#[derive(Clone, Debug, PartialEq)]
struct StaticExpression {
    op: StaticOp,
    left: Box<Step>,
    right: Value,
}

#[derive(Clone, Debug, PartialEq)]
struct Bjorklund {
    left: Box<Step>,
    steps: Box<Step>,
    pulses: Box<Step>,
    rotation: Option<Box<Step>>,
}

#[derive(Clone, Debug, PartialEq)]
struct Range {
    start: i32,
    end: i32,
}

// -------------------------------------------------------------------------------------------------

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

#[cfg(test)]
impl Display for Pitch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = match self.note {
            0 => "c",
            1 => "c#",
            2 => "d",
            3 => "d#",
            4 => "e",
            5 => "f",
            6 => "f#",
            7 => "g",
            8 => "g#",
            9 => "a",
            10 => "a#",
            11 => "b",
            _ => "",
        };
        if self.octave == 4 {
            f.write_str(n)
        } else {
            f.write_fmt(format_args!("{}{}", n, self.octave))
        }
    }
}

impl Value {
    fn to_target(&self) -> Target {
        match &self {
            Value::Rest => Target::None,
            Value::Hold => Target::None,
            Value::Name(n) => Target::Name(n.clone()),
            Value::Integer(i) => Target::Index(*i),
            Value::Float(f) => Target::Index(*f as i32),
            Value::Pitch(p) => Target::Name(format!("{:?}", p)), // TODO might not be the best conversion idea
        }
    }
    fn to_integer(&self) -> Option<i32> {
        match &self {
            Value::Rest => None,
            Value::Hold => None,
            Value::Name(_n) => None,
            Value::Integer(i) => Some(*i),
            Value::Float(f) => Some(*f as i32),
            Value::Pitch(n) => Some(n.note as i32),
        }
    }

    fn to_float(&self) -> Option<f64> {
        match &self {
            Value::Rest => None,
            Value::Hold => None,
            Value::Name(_n) => None,
            Value::Integer(i) => Some(*i as f64),
            Value::Float(f) => Some(*f),
            Value::Pitch(n) => Some(n.note as f64),
        }
    }

    fn to_chance(&self) -> Option<f64> {
        match &self {
            Value::Rest => None,
            Value::Hold => None,
            Value::Name(_n) => None,
            Value::Integer(i) => Some((*i as f64).clamp(0.0, 100.0) / 100.0),
            Value::Float(f) => Some(f.clamp(0.0, 1.0)),
            Value::Pitch(n) => Some((n.note as f64).clamp(0.0, 128.0) / 128.0),
        }
    }
}

#[cfg(test)]
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

    /// transforms the span to 0..1 based on an outer span
    /// assumes self is inside outer
    fn normalize(&mut self, outer: &Span) {
        let outer_length = outer.length();
        self.start = (self.start - outer.start) / outer_length;
        self.end = (self.end - outer.start) / outer_length;
    }

    fn whole_range(&self) -> std::ops::Range<u32> {
        let start = self.start.floor().to_u32().unwrap_or_default();
        let end = self.end.ceil().to_u32().unwrap_or_default();
        start..end
    }

    // fn overlaps(&self, span: &Span) -> bool {
    //     (self.start <= span.start && span.start < self.end) || (self.start < span.end && span.end <= self.end)
    // }

    fn includes(&self, span: &Span) -> bool {
        self.start <= span.start && span.start < self.end
    }

    /// Limit self to not extend beyond the target span
    /// this function assumes self.overlaps(span) is true
    fn crop(&mut self, span: &Span) {
        if self.start < span.start {
            self.start = span.start;
        }
        if self.end > span.end {
            self.end = span.end
        }
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

impl Event {
    #[cfg(test)]
    fn at(start: Fraction, length: Fraction) -> Self {
        Self {
            length,
            span: Span {
                start,
                end: start + length,
            },
            string: "~".to_string(),
            value: Value::Rest,
            target: Target::None,
        }
    }

    #[cfg(test)]
    fn with_note(&self, note: u8, octave: u8) -> Self {
        let pitch = Pitch { note, octave };
        Self {
            value: Value::Pitch(pitch.clone()),
            string: pitch.to_string(),
            ..self.clone()
        }
    }

    #[cfg(test)]
    fn with_int(&self, i: i32) -> Self {
        Self {
            value: Value::Integer(i),
            string: i.to_string(),
            ..self.clone()
        }
    }

    #[cfg(test)]
    fn with_name(&self, n: &'static str) -> Self {
        Self {
            value: Value::Name(String::from(n)),
            string: n.to_string(),
            ..self.clone()
        }
    }

    #[cfg(test)]
    fn with_float(&self, f: f64) -> Self {
        Self {
            value: Value::Float(f),
            string: f.to_string(),
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
}

impl PartialEq<Event> for Event {
    fn eq(&self, other: &Event) -> bool {
        // Don't compare self.string: compare interpreted values and target only.
        self.length == other.length
            && self.span == other.span
            && self.value == other.value
            && self.target == other.target
    }
}

#[cfg(test)]
impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{:.3} -> {:.3} | {:?} {}",
            self.span.start, self.span.end, self.value, self.target
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
            string: "~".to_string(),
            value: Value::Rest,
            target: Target::None,
        })
    }

    /// Fits a list of events into a Span of 0..1
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

    fn filter_mut<F>(&mut self, predicate: &mut F) -> bool
    where
        F: FnMut(&mut Event) -> bool,
    {
        match self {
            Events::Multi(m) => {
                let mut filtered = vec![];
                for e in &mut m.events {
                    match e {
                        Events::Single(s) => {
                            if predicate(s) {
                                filtered.push(e.clone())
                            }
                        }
                        _ => {
                            if e.filter_mut(predicate) {
                                filtered.push(e.clone())
                            }
                        }
                    }
                }
                m.events = filtered;
                !m.events.is_empty()
            }
            Events::Poly(p) => {
                let mut filtered = vec![];
                for e in &mut p.channels {
                    if e.filter_mut(predicate) {
                        filtered.push(e.clone())
                    }
                }
                p.channels = filtered;
                !p.channels.is_empty()
            }
            Events::Single(_) => true,
        }
    }

    fn crop(&mut self, span: &Span) {
        self.filter_mut(&mut |e| {
            if span.includes(&e.span) {
                e.span.crop(span);
                true
            } else {
                false
            }
        });
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

    /// recursively transform the spans of events from 0..1 to a given span
    fn transform_spans(&mut self, span: &Span) {
        let unit = span.length();
        match self {
            Events::Single(s) => {
                s.length *= unit;
                s.span = s.span.transform(span);
            }
            Events::Multi(m) => {
                m.length *= unit;
                m.span = m.span.transform(span);

                for e in &mut m.events {
                    e.transform_spans(&m.span);
                }
            }
            Events::Poly(p) => {
                p.length *= unit;
                p.span = p.span.transform(span);
                for e in &mut p.channels {
                    e.transform_spans(&p.span);
                }
            }
        }
    }

    /// recursively transform the spans of events to 0..1 range
    fn normalize_spans(&mut self, span: &Span) {
        match self {
            Events::Single(s) => {
                s.span.normalize(span);
                s.length = s.span.length();
            }
            Events::Multi(m) => {
                for e in &mut m.events {
                    e.normalize_spans(&m.span);
                }

                m.span.normalize(span);
                m.length = m.span.length();
            }
            Events::Poly(p) => {
                for e in &mut p.channels {
                    e.normalize_spans(span);
                }
                p.span.normalize(span);
                p.length = p.span.length();
            }
        }
    }

    /// Recursively collapses Multi and Poly Events into vectors of Single Events
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
        let mut result: Vec<Event> = Vec::with_capacity(events.len());
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
        let mut result: Vec<Event> = Vec::with_capacity(events.len());
        for e in events {
            match e.value {
                Value::Rest => {
                    if let Some(last) = result.last_mut() {
                        match last.value {
                            Value::Rest => last.extend(e),
                            _ => result.push(e.clone()),
                        }
                    }
                }
                _ => result.push(e.clone()),
            }
        }
        result
    }

    /// Removes Holds by extending preceding events and filters out Rests
    fn merge(&self, channels: &mut [Vec<Event>]) {
        for events in &mut *channels {
            *events = Self::merge_holds(events);
        }
        for events in channels {
            *events = Self::merge_rests(events);
        }
    }

    fn export(&mut self) -> Vec<Vec<Event>> {
        let mut channels = vec![];
        self.flatten(&mut channels, 0);
        self.merge(&mut channels);

        #[cfg(test)]
        {
            println!("\nOUTPUT {}", 0);
            // println!("\nOUTPUT {}", self.iteration);
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

        channels
    }

    #[cfg(test)]
    #[allow(dead_code)] // TODO remove this once the "step * step" is done
    fn print(&self) {
        match self {
            Events::Single(s) => println!("[{}] {}", s.length, s),
            Events::Multi(m) => {
                println!("multi {} -> {} [{}]", m.span.start, m.span.end, m.length);
                for e in &m.events {
                    e.print()
                }
            }
            Events::Poly(p) => {
                println!("multi {} -> {} [{}]", p.span.start, p.span.end, p.length);
                for e in &p.channels {
                    e.print()
                }
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Parser)]
#[grammar = "tidal/cycle.pest"]
struct CycleParser {}

/// the errors here should be unreachable unless there is a bug in the pest grammar
impl CycleParser {
    /// recursively parse a pair as a Step
    fn step(pair: Pair<Rule>) -> Result<Step, String> {
        match pair.as_rule() {
            Rule::single => Self::single(pair),
            Rule::repeat => Ok(Step::Repeat),
            Rule::subdivision | Rule::mini => Self::group(pair, Step::subdivision),
            Rule::alternating => Self::group(pair, Step::alternating),
            Rule::polymeter => Self::polymeter(pair),
            Rule::range => Self::range(pair),
            Rule::expression => Self::expression(pair),
            Rule::stack | Rule::section | Rule::choices | Rule::split => {
                // stacks can only appear inside rules for Subdivision, Alternating or Polymeter
                // sections, choices and splits are always immediately handled within other rules
                Err(format!("unexpected pair\n{:?}", pair))
            }
            _ => Err(format!("rule not implemented\n{:?}", pair)),
        }
    }

    /// parse a pair inside a single as a value
    fn value(pair: Pair<Rule>) -> Result<Value, String> {
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

    fn single(pair: Pair<Rule>) -> Result<Step, String> {
        pair.clone()
            .into_inner()
            .next()
            .ok_or_else(|| format!("empty single {}", pair))
            .and_then(|value_pair| {
                Ok(Step::Single(Single {
                    string: value_pair.as_str().to_string(),
                    value: Self::value(value_pair)?,
                }))
            })
    }

    /// transform static steps into their final form and push them onto a list
    fn push_applied(steps: &mut Vec<Step>, step: Step) {
        match &step {
            Step::Repeat => {
                let repeat = steps.last().cloned().unwrap_or(Step::rest());
                steps.push(repeat)
            }
            Step::StaticExpression(e) => match e.op {
                StaticOp::Replicate() => {
                    steps.push(e.left.as_ref().clone());
                    if let Some(repeats) = e.right.to_integer() {
                        if repeats > 0 {
                            for _i in 1..repeats {
                                steps.push(e.left.as_ref().clone())
                            }
                        }
                    }
                }
                StaticOp::Weight() => {
                    steps.push(e.left.as_ref().clone());
                    if let Some(repeats) = e.right.to_integer() {
                        if repeats > 0 {
                            for _i in 1..repeats {
                                steps.push(Step::Single(Single {
                                    value: Value::Hold,
                                    string: "_".to_string(),
                                }))
                            }
                        }
                    }
                }
                _ => steps.push(step),
            },
            Step::Range(r) => {
                let range = if r.start <= r.end {
                    Box::new(r.start..=r.end) as Box<dyn Iterator<Item = i32>>
                } else {
                    Box::new((r.end..=r.start).rev()) as Box<dyn Iterator<Item = i32>>
                };
                for i in range {
                    steps.push(Step::Single(Single {
                        value: Value::Integer(i),
                        string: i.to_string(),
                    }))
                }
            }
            _ => steps.push(step),
        }
    }

    fn section(pair: Pair<Rule>) -> Result<Vec<Step>, String> {
        let mut steps: Vec<Step> = vec![];
        for pair in pair.into_inner() {
            let step = Self::step(pair)?;
            Self::push_applied(&mut steps, step);
        }
        Ok(steps)
    }

    fn split(pair: Pair<Rule>) -> Result<Vec<Step>, String> {
        pair.into_inner()
            .map(|p| Self::section(p).map(Step::subdivision))
            .collect()
    }

    // stacks will always have a non-empty list of sections or splits inside them
    fn stack(pair: Pair<Rule>) -> Result<Vec<Vec<Step>>, String> {
        pair.into_inner()
            .map(|p| match p.as_rule() {
                Rule::section => Self::section(p),
                Rule::split => Self::split(p),
                _ => Err(format!("unexpected rule in stack\n{:?}", p)),
            })
            .collect()
    }

    fn choices(pair: Pair<Rule>) -> Result<Vec<Step>, String> {
        let mut choices: Vec<Step> = vec![];
        for p in pair.clone().into_inner() {
            let step = p
                .into_inner()
                .next()
                .ok_or_else(|| format!("empty choice\n{:?}", pair))?;
            let choice = Self::step(step)?;
            Self::push_applied(&mut choices, choice)
        }
        Ok(vec![Step::Choices(Choices { choices })])
    }

    // helper to convert a section or single to a vector of Steps
    fn inner_steps(pair: Pair<Rule>) -> Result<Vec<Step>, String> {
        let inner = pair
            .clone()
            .into_inner()
            .next()
            .ok_or_else(|| format!("empty section\n{:?}", pair))?;
        match inner.as_rule() {
            Rule::repeat => Ok(vec![Step::Repeat]),
            Rule::single => {
                let single = Self::single(inner)?;
                Ok(vec![single])
            }
            Rule::split => Self::split(inner),
            Rule::section => Self::section(inner),
            Rule::choices => Self::choices(inner),
            _ => Err(format!("unexpected rule in section\n{:?}", inner)),
        }
    }

    // TODO allow for polymeter count other than single
    fn polymeter_tail(pair: Pair<Rule>) -> Result<Step, String> {
        if let Some(count) = pair.clone().into_inner().next() {
            Self::step(count).and_then(|step| match &step {
                Step::Single(s) => match s.value {
                    Value::Integer(_) | Value::Float(_) => Ok(step),
                    _ => Err(format!("invalid multiplier: {:?}", step)),
                },
                _ => Err(format!("invalid multiplier: {:?}", step)),
            })
        } else {
            Err(format!("missing polymeter count '{}'", pair.as_str()))
        }
    }

    fn polymeter(pair: Pair<Rule>) -> Result<Step, String> {
        let mut count: Option<Step> = None;
        let mut steps: Option<Vec<Step>> = None;
        let mut stack: Option<Vec<Vec<Step>>> = None;

        for p in pair.clone().into_inner() {
            match p.as_rule() {
                Rule::polymeter_tail => count = Self::polymeter_tail(p).ok(),
                Rule::stack => stack = Self::stack(p).ok(),
                Rule::split => steps = Self::split(p).ok(),
                _ => steps = Self::section(p).ok(),
            }
        }

        match (count, stack, steps) {
            (Some(count), None, Some(steps)) => {
                // a regular polymeter with explicit count
                Ok(Step::polymeter(steps, count))
            }
            (Some(count), Some(stack), _) => {
                // sections in a stack with explicit count will all have that
                Ok(Step::Stack(Stack {
                    stack: stack
                        .into_iter()
                        .map(|steps| Step::polymeter(steps, count.clone()))
                        .collect(),
                }))
            }
            (None, Some(stack), _) => {
                let count = stack
                    .first()
                    .map(Vec::len)
                    .ok_or_else(|| format!("empty stack {:?}", stack))?;

                if stack.len() > 1 && count > 0 {
                    let count = Step::Single(Single {
                        value: Value::Integer(count as i32),
                        string: count.to_string(),
                    });
                    // if there is a stack but no count, the first section will determine the count of the rest
                    Ok(Step::Stack(Stack {
                        stack: stack
                            .into_iter()
                            .map(|steps| Step::polymeter(steps, count.clone()))
                            .collect(),
                    }))
                } else {
                    // unreachable, a stack will always have more than one sections with each having at least one item
                    Err(format!("invalid stack {:?}", stack))
                }
            }
            // if there is only one section and no count, it is treated as a subdivision
            (None, None, Some(steps)) => Ok(Step::subdivision(steps)),
            // empty polymeter like {} and {}%2 will become a single rest
            _ => Ok(Step::rest()),
        }
    }

    // helper to parse subdivision and alternating groups
    fn group(pair: Pair<Rule>, fun: fn(Vec<Step>) -> Step) -> Result<Step, String> {
        if let Some(first) = pair.clone().into_inner().next() {
            match first.as_rule() {
                Rule::stack => Self::stack(first).map(|channels| {
                    Step::Stack(Stack {
                        stack: channels.into_iter().map(fun).collect(),
                    })
                }),
                _ => Ok(fun(Self::inner_steps(pair)?)),
            }
        } else {
            Ok(Step::rest())
        }
    }

    fn range(pair: Pair<Rule>) -> Result<Step, String> {
        let mut inner = pair.clone().into_inner();
        let start_pair = inner
            .next()
            .ok_or_else(|| format!("empty expression\n{:?}", pair))?;
        let start = start_pair.as_str().parse::<i32>().map_err(|_| {
            format!(
                "range expected integer on the left side, got '{}'",
                start_pair.as_str()
            )
        })?;

        let end_pair = inner
            .next()
            .ok_or_else(|| "range expression has no right side".to_string())?;
        let end = end_pair.as_str().parse::<i32>().map_err(|_| {
            format!(
                "range expected integer on the right side, got '{}'",
                end_pair.as_str()
            )
        })?;
        Ok(Step::Range(Range { start, end }))
    }

    fn bjorklund(left: Step, op_pair: Pair<Rule>) -> Result<Step, String> {
        let mut inner = op_pair.clone().into_inner();

        let steps = inner
            .next()
            .ok_or_else(|| format!("no steps in bjorklund\n{:?}", op_pair))
            .and_then(Self::step)?;

        let pulses = inner
            .next()
            .ok_or_else(|| format!("no pulse in bjorklund\n{:?}", op_pair))
            .and_then(Self::step)?;

        let rotate = inner.next().map(Self::step).transpose()?;

        Ok(Step::Bjorklund(Bjorklund {
            left: Box::new(left),
            pulses: Box::new(pulses),
            steps: Box::new(steps),
            rotation: rotate.map(Box::new),
        }))
    }

    fn speed_expression(left: Step, op: DynamicOp, op_pair: Pair<Rule>) -> Result<Step, String> {
        let mut inner = op_pair.into_inner();
        let right = inner
            .next()
            .ok_or_else(|| format!("missing right hand side in expression\n{:?}", inner))
            .and_then(Self::step)?;
        match right {
            Step::Single(_) => Ok(Step::DynamicExpression(DynamicExpression {
                left: Box::new(left),
                right: Box::new(right),
                op,
            })),
            _ => Err("only single values are supported on the right hand side".to_string()),
        }
    }

    fn static_expression(left: Step, op: StaticOp, pair: Pair<Rule>) -> Result<Step, String> {
        let right = if let Some(right_pair) = pair.into_inner().next() {
            let value = right_pair
                .clone()
                .into_inner()
                .next()
                .ok_or_else(|| format!("invalid right hand {:?}", right_pair))
                .and_then(Self::value)?;
            if matches!(op, StaticOp::Target()) && matches!(value, Value::Pitch(_)) {
                Value::Name(right_pair.as_str().to_string())
            } else {
                value
            }
        } else {
            op.default_value()
        };

        Ok(Step::StaticExpression(StaticExpression {
            left: Box::new(left),
            right,
            op,
        }))
    }

    fn expression(pair: Pair<Rule>) -> Result<Step, String> {
        let mut inner = pair.clone().into_inner();

        let left = inner
            .next()
            .ok_or_else(|| format!("empty expression\n{:?}", pair))
            .and_then(Self::step)?;

        let op_pair = inner
            .next()
            .ok_or_else(|| format!("incomplete expression\n{:?}", pair))?;

        match Operator::parse(op_pair.clone())? {
            Operator::Static(op) => Self::static_expression(left, op, op_pair),
            Operator::Dynamic(op) => match op {
                DynamicOp::Bjorklund() => Self::bjorklund(left, op_pair),
                _ => Self::speed_expression(left, op, op_pair),
            },
        }
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
struct CycleState {
    iteration: u32,
    rng: Xoshiro256PlusPlus,
    step: u32,
    events: usize,
}

impl Cycle {
    fn output_span(
        step: &Step,
        state: &mut CycleState,
        span: &Span,
        limit: usize,
    ) -> Result<Events, String> {
        let range = span.whole_range();
        let mut cycles = vec![];
        for cycle in range {
            let mut events = Self::output(step, state, cycle, limit)?;
            events.transform_spans(&Span::new(Fraction::from(cycle), Fraction::from(cycle + 1)));
            cycles.push(events)
        }
        let mut events = Events::Multi(MultiEvents {
            span: span.clone(),
            length: span.length(),
            events: cycles,
        });
        events.crop(span);
        Ok(events)
    }

    fn output_multiplied(
        step: &Step,
        state: &mut CycleState,
        cycle: u32,
        mult: Fraction,
        limit: usize,
    ) -> Result<Events, String> {
        let span = Span::new(
            Fraction::from(cycle) * mult,
            Fraction::from(cycle + 1) * mult,
        );
        let mut events = Self::output_span(step, state, &span, limit)?;
        events.normalize_spans(&span);
        Ok(events)
    }

    // recursively output events for the entire cycle based on some state (random seed)
    fn output(
        step: &Step,
        state: &mut CycleState,
        cycle: u32,
        limit: usize,
    ) -> Result<Events, String> {
        let events = match step {
            // repeats only make it here if they had no preceding value
            Step::Repeat => Events::empty(),
            // ranges get applied at parse time
            Step::Range(_) => Events::empty(),
            Step::Single(s) => {
                state.events += 1;
                if state.events > limit {
                    return Err(format!(
                        "the cycle's event limit of {} was exceeded!",
                        limit
                    ));
                }
                Events::Single(Event {
                    length: Fraction::one(),
                    target: Target::None,
                    span: Span::default(),
                    string: s.string.clone(),
                    value: s.value.clone(),
                })
            }
            Step::Subdivision(sd) => {
                if sd.steps.is_empty() {
                    Events::empty()
                } else {
                    let mut events = vec![];
                    for s in &sd.steps {
                        let e = Self::output(s, state, cycle, limit)?;
                        events.push(e)
                    }

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
                    let length = a.steps.len() as u32;
                    let current = cycle % length;
                    if let Some(step) = a.steps.get(current as usize) {
                        Self::output(step, state, cycle / length, limit)?
                    } else {
                        Events::empty() // unreachable
                    }
                }
            }
            Step::Choices(cs) => {
                let choice = state.rng.gen_range(0..cs.choices.len());
                Self::output(&cs.choices[choice], state, cycle, limit)? // TODO seed the rng properly
            }
            Step::Polymeter(pm) => {
                let step = pm.steps.as_ref();
                let length = pm.length();
                let count: usize = match pm.count.as_ref() {
                    Step::Single(s) => s.value.to_integer().unwrap_or(1) as usize,
                    _ => 1,
                };
                let mult = Fraction::from(count) / Fraction::from(length);
                Self::output_multiplied(step, state, cycle, mult, limit)?
            }
            Step::Stack(st) => {
                if st.stack.is_empty() {
                    Events::empty()
                } else {
                    let mut channels = vec![];
                    for s in &st.stack {
                        channels.push(Self::output(s, state, cycle, limit)?)
                    }
                    Events::Poly(PolyEvents {
                        span: Span::default(),
                        length: Fraction::one(),
                        channels,
                    })
                }
            }
            Step::StaticExpression(e) => {
                match e.op {
                    StaticOp::Target() => {
                        let mut out = Self::output(e.left.as_ref(), state, cycle, limit)?;
                        out.mutate_events(&mut |event| event.target = e.right.to_target());
                        out
                    }
                    StaticOp::Degrade() => {
                        let mut out = Self::output(e.left.as_ref(), state, cycle, limit)?;
                        out.mutate_events(&mut |event: &mut Event| {
                            if let Some(chance) = e.right.to_chance() {
                                // TODO seed the rng properly
                                if chance < state.rng.gen_range(0.0..1.0) {
                                    event.value = Value::Rest
                                }
                            }
                        });
                        out
                    }
                    _ => {
                        // unreachable, these expressions were immediately applied in Self::push_applied
                        Events::empty()
                    }
                }
            }
            Step::DynamicExpression(e) => {
                #[allow(clippy::single_match)]
                // TODO support something other than Step::Single as the right hand side
                match e.right.as_ref() {
                    Step::Single(s) => {
                        if let Some(right) = s.value.to_float() {
                            let mult = Fraction::from(if e.op == DynamicOp::Slow() {
                                1.0 / right
                            } else {
                                right
                            });
                            Self::output_multiplied(e.left.as_ref(), state, cycle, mult, limit)?
                        } else {
                            Events::empty()
                        }
                    }
                    _ => Events::empty(),
                }
            }
            Step::Bjorklund(b) => {
                let mut events = vec![];
                #[allow(clippy::single_match)]
                // TODO support something other than Step::Single as the right hand side
                match b.pulses.as_ref() {
                    Step::Single(pulses_single) => {
                        match b.steps.as_ref() {
                            Step::Single(steps_single) => {
                                let rotation = match &b.rotation {
                                    None => None,
                                    Some(r) => match r.as_ref() {
                                        Step::Single(rotation_single) => {
                                            rotation_single.value.to_integer()
                                        }
                                        _ => None, // TODO support something other than Step::Single as rotation
                                    },
                                };
                                if let Some(steps) = steps_single.value.to_integer() {
                                    if let Some(pulses) = pulses_single.value.to_integer() {
                                        let out =
                                            Self::output(b.left.as_ref(), state, cycle, limit)?;
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
            }
        };
        Ok(events)
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
            Step::Range(r) => format!("Range {}..{}", r.start, r.end),
            Step::Single(s) => match &s.value {
                Value::Pitch(_p) => format!("{:?} {}", s.value, s.string),
                _ => format!("{:?} {:?}", s.value, s.string),
            },
            Step::Subdivision(sd) => format!("Subdivision [{}]", sd.steps.len()),
            Step::Alternating(a) => format!("Alternating <{}>", a.steps.len()),
            Step::Polymeter(pm) => format!("Polymeter {{{}}}", pm.length()), //, pm.count),
            Step::Choices(cs) => format!("Choices |{}|", cs.choices.len()),
            Step::Stack(st) => format!("Stack ({})", st.stack.len()),
            Step::DynamicExpression(e) => format!("Expression {:?}", e.op),
            Step::StaticExpression(e) => {
                format!("SingleExpression {:?} : {:?}", e.op, e.right)
            }
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
            assert_eq!(cycle.generate()?, out);
        }
        Ok(())
    }

    fn assert_cycle_equality(a: &str, b: &str) -> Result<(), String> {
        assert_eq!(
            Cycle::from(a, None)?.generate()?,
            Cycle::from(b, None)?.generate()?,
        );
        Ok(())
    }

    #[test]

    pub fn cycle() -> Result<(), String> {
        assert_eq!(
            Cycle::from("a b c d", None)?.generate()?,
            [[
                Event::at(F::from(0), F::new(1u8, 4u8)).with_note(9, 4),
                Event::at(F::new(1u8, 4u8), F::new(1u8, 4u8)).with_note(11, 4),
                Event::at(F::new(2u8, 4u8), F::new(1u8, 4u8)).with_note(0, 4),
                Event::at(F::new(3u8, 4u8), F::new(1u8, 4u8)).with_note(2, 4),
            ]]
        );
        assert_eq!(
            Cycle::from("\ta\r\n\tb\nc\n d\n\n", None)?.generate()?,
            Cycle::from("a b c d", None)?.generate()?
        );
        assert_eq!(
            Cycle::from("a b [ c d ]", None)?.generate()?,
            [[
                Event::at(F::from(0), F::new(1u8, 3u8)).with_note(9, 4),
                Event::at(F::new(1u8, 3u8), F::new(1u8, 3u8)).with_note(11, 4),
                Event::at(F::new(2u8, 3u8), F::new(1u8, 6u8)).with_note(0, 4),
                Event::at(F::new(5u8, 6u8), F::new(1u8, 6u8)).with_note(2, 4),
            ]]
        );
        assert_eq!(
            Cycle::from("[a a] [b4 b5 b6] [c0 d1 c2 d3]", None)?.generate()?,
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
            Cycle::from("[a0 [bb1 [b2 c3]]] c#4 [[[d5 D#6] E7 ] F8]", None)?.generate()?,
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
            Cycle::from("[R [e [n o]]] , [[[i s] e ] _]", None)?.generate()?,
            vec![
                vec![
                    Event::at(F::from(0), F::new(1u8, 2u8)).with_name("R"),
                    Event::at(F::new(1u8, 2u8), F::new(1u8, 4u8)).with_note(4, 4),
                    Event::at(F::new(3u8, 4u8), F::new(1u8, 8u8)).with_name("n"),
                    Event::at(F::new(7u8, 8u8), F::new(1u8, 8u8)).with_name("o"),
                ],
                vec![
                    Event::at(F::from(0), F::new(1u8, 8u8)).with_name("i"),
                    Event::at(F::new(1u8, 8u8), F::new(1u8, 8u8)).with_name("s"),
                    Event::at(F::new(1u8, 4u8), F::new(3u8, 4u8)).with_note(4, 4),
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
                    Event::at(F::new(1u8, 2u8), F::new(1u8, 2u8)).with_note(0, 4)
                ]],
                vec![vec![
                    Event::at(F::new(1u8, 2u8), F::new(1u8, 2u8)).with_note(11, 4)
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
            Cycle::from("[1 middle _] {}%42 [] <>", None)?.generate()?,
            [[
                Event::at(F::from(0), F::new(1u8, 12u8)).with_int(1),
                Event::at(F::new(1u8, 12u8), F::new(1u8, 6u8)).with_name("middle"),
                Event::at(F::new(1u8, 4u8), F::new(3u8, 4u8)),
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
            Cycle::from("1 second*2 eb3*3 [32 32]*4", None)?.generate()?,
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
                    Event::at(F::new(1u8, 11u8), F::new(2u8, 11u8)),
                    Event::at(F::new(3u8, 11u8), F::new(1u8, 11u8)).with_name("outside"),
                    Event::at(F::new(4u8, 11u8), F::new(2u8, 11u8)),
                    Event::at(F::new(6u8, 11u8), F::new(1u8, 11u8)).with_name("outside"),
                    Event::at(F::new(7u8, 11u8), F::new(2u8, 11u8)),
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
                    Event::at(F::new(3u8, 5u8), F::new(2u8, 5u8)),
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
                    Event::at(F::new(3u8, 5u8), F::new(2u8, 5u8)),
                ]],
            ],
        )?;

        assert_eq!(
            Cycle::from("1!2 3 [4!3 5]", None)?.generate()?,
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

        assert_cycles(
            "[0 1]/2",
            vec![
                vec![vec![Event::at(F::from(0), F::new(1u8, 1u8)).with_int(0)]],
                vec![vec![Event::at(F::from(0), F::new(1u8, 1u8)).with_int(1)]],
            ],
        )?;

        assert_cycles(
            "[0 1]*2.5",
            vec![
                vec![vec![
                    Event::at(F::from(0), F::new(1u8, 5u8)).with_int(0),
                    Event::at(F::new(1u8, 5u8), F::new(1u8, 5u8)).with_int(1),
                    Event::at(F::new(2u8, 5u8), F::new(1u8, 5u8)).with_int(0),
                    Event::at(F::new(3u8, 5u8), F::new(1u8, 5u8)).with_int(1),
                    Event::at(F::new(4u8, 5u8), F::new(1u8, 5u8)).with_int(0),
                ]],
                vec![vec![
                    Event::at(F::from(0), F::new(1u8, 5u8)).with_int(1),
                    Event::at(F::new(1u8, 5u8), F::new(1u8, 5u8)).with_int(0),
                    Event::at(F::new(2u8, 5u8), F::new(1u8, 5u8)).with_int(1),
                    Event::at(F::new(3u8, 5u8), F::new(1u8, 5u8)).with_int(0),
                    Event::at(F::new(4u8, 5u8), F::new(1u8, 5u8)).with_int(1),
                ]],
            ],
        )?;

        assert_eq!(
            Cycle::from("c(3,8,9)", None)?.generate()?,
            [[
                Event::at(F::new(2u8, 8u8), F::new(1u8, 8u8)).with_note(0, 4),
                Event::at(F::new(3u8, 8u8), F::new(1u8, 4u8)),
                Event::at(F::new(5u8, 8u8), F::new(1u8, 8u8)).with_note(0, 4),
                Event::at(F::new(6u8, 8u8), F::new(1u8, 8u8)),
                Event::at(F::new(7u8, 8u8), F::new(1u8, 8u8)).with_note(0, 4),
            ]]
        );

        assert_eq!(
            Cycle::from("a? b?", None)?.root,
            Cycle::from("a?0.5 b?0.5", None)?.root
        );

        assert_cycle_equality("[a b c](3,8,9)", "[a b c](3,8,1)")?;

        assert_cycle_equality("[a b c](3,8,7)", "[a b c](3,8,-1)")?;

        assert_cycle_equality("[a a a a]", "[a ! ! !]")?;

        assert_cycle_equality("[! ! a !]", "[~ ~ a a]")?;

        assert_cycle_equality("a ~ ~ ~", "a - - -")?;

        assert_cycle_equality("[a b] ! ! <a b c> !", "[a b] [a b] [a b] <a b c> <a b c>")?;

        assert_cycle_equality("{a b!2 c}%3", "{a b b c}%3")?;

        assert_cycle_equality("a b, {c d e}%2", "{a b, c d e}")?;

        assert_cycle_equality("0..3", "0 1 2 3")?;

        assert_cycle_equality("-5..-8", "-5 -6 -7 -8")?;

        assert_cycle_equality("a b . c d", "[a b] [c d]")?;

        assert_cycle_equality(
            "a b . c d e . f g h i [j k . l m]",
            "[a b] [c d e] [f g h i [[j k] [l m]]]",
        )?;

        assert_cycle_equality(
            "a b . c d e , f g h i . j k, l m",
            "[a b] [c d e], [[f g h i] [j k]], [l m]",
        )?;

        assert_cycle_equality("{a b . c d . f g h}%2", "{[a b] [c d] [f g h]}%2")?;

        assert_cycle_equality("<a b . c d . f g h>", "<[a b] [c d] [f g h]>")?;

        assert_cycles(
            "[0 1 2]/2",
            vec![
                vec![vec![
                    Event::at(F::from(0), F::new(2u8, 3u8)).with_int(0),
                    Event::at(F::new(2u8, 3u8), F::new(1u8, 3u8)).with_int(1),
                ]],
                vec![vec![
                    Event::at(F::new(1u8, 3u8), F::new(2u8, 3u8)).with_int(2)
                ]],
                vec![vec![
                    Event::at(F::from(0), F::new(2u8, 3u8)).with_int(0),
                    Event::at(F::new(2u8, 3u8), F::new(1u8, 3u8)).with_int(1),
                ]],
            ],
        )?;

        assert!(Span::new(F::new(0u8, 1u8), F::new(1u8, 1u8))
            .includes(&Span::new(F::new(1u8, 2u8), F::new(2u8, 1u8))));

        // TODO test random outputs // parse_with_debug("[a b c d]?0.5");

        assert!(Cycle::from("[[a b c d]*100]*100", None)?
            .generate()
            .is_err());
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
        Ok(())
    }
}
