#![allow(dead_code)]
use core::fmt::Display;
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use rand::{thread_rng, Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;
// use std::fs;
// use pest::iterators::Pairs;
// use pest::Token::Start;
// use pest::error::ErrorVariant;

#[derive(Parser)]
#[grammar = "rhythm/tidal.pest"]
struct MiniParser {
}

#[derive(Clone)]
struct State {
    seed: Option<[u8; 32]>,
}

#[derive(Clone, Debug)]
struct Pitch {
    note: u8,
    octave: u8,
}

impl Pitch {
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

    fn parse(pair: Pair<Rule>) -> Pitch {
        let mut pitch = Pitch {
            note: 0,
            octave: 4,
        };
        let mut mark: i8 = 0;
        for p in pair.into_inner() {
            match p.as_rule() {
                Rule::note => {
                    if let Some(c) = String::from(p.as_str()).to_ascii_lowercase().chars().next() {
                        pitch.note = Pitch::as_note_value(c).unwrap_or(pitch.note)
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
                pitch.note = 11 as u8;
            }
        }else if pitch.note == 11 && mark == 1{
            if pitch.octave < 10 {
                pitch.note = 0;
                pitch.octave += 1;
            }
        }else{
            pitch.note = ((pitch.note as i8) + mark) as u8;
        }
        // pitch.note = pitch.note.clamp(0, 127);
        pitch
    }

}

#[derive(Clone, Debug)]
enum Step {
    Single(Single),
    Alternating(Alternating),
    Subdivision(Subdivision),
    Polymeter(Polymeter),
    Stack(Stack),
    Choices(Choices),
    Expression(Expression),
    Bjorklund(Bjorklund),
}

#[derive(Clone, Debug, Default)]
struct Single {
    value: StepValue,
    string: String,
}

impl Single {
    fn default() -> Self {
        Single {
            value: StepValue::Rest,
            string: String::from("~")
        }
    }
    fn to_integer(&self) -> Option<i32> {
        match &self.value {
            StepValue::Rest => None,
            StepValue::Hold => None,
            StepValue::Name(_n) => None,
            StepValue::Integer(i) => Some(*i),
            StepValue::Float(f) => Some(*f as i32),
            StepValue::Pitch(n) => Some(n.note as i32),
        }
    }
    fn to_target(&self) -> Target {
        match &self.value {
            StepValue::Rest => Target::None,
            StepValue::Hold => Target::None,
            StepValue::Name(n) => Target::Name(n.clone()),
            StepValue::Integer(i) => Target::Index(*i),
            StepValue::Float(f) => Target::Index(*f as i32),
            StepValue::Pitch(_n) => Target::Name(self.string.clone()), // TODO might not be the best conversion idea
        }
    }
    fn to_chance(&self) -> Option<f64> {
        match &self.value {
            StepValue::Rest => None,
            StepValue::Hold => None,
            StepValue::Name(_n) => None,
            StepValue::Integer(i) => Some((*i as f64).clamp(0.0, 100.0) / 100.0),
            StepValue::Float(f) => Some(f.clamp(0.0, 1.0)),
            StepValue::Pitch(n) => Some((n.note as f64).clamp(0.0, 128.0) / 128.0),
        }
    }
}

#[derive(Clone, Debug)]
struct Alternating {
    current: usize,
    steps: Vec<Step>,
}

#[derive(Clone, Debug)]
struct Subdivision {
    steps: Vec<Step>,
}

#[derive(Clone, Debug)]
struct Polymeter {
    count: usize,
    offset: usize,
    steps: Vec<Step>,
}

#[derive(Clone, Debug)]
struct Choices {
    choices: Vec<Step>,
}

#[derive(Clone, Debug)]
struct Stack {
    stack: Vec<Step>,
}

#[derive(Clone, Debug)]
enum Operator {
    Fast(),       // *
    Target(),     // :
    Degrade(),    // ?
    Replicate(),  // !
    // Weight(),     // @
    // Slow(),       // /
}
impl Operator {
    fn parse(pair: Pair<Rule>) -> Result<Operator, &str> {
        match pair.as_rule() {
            Rule::op_fast => Ok(Operator::Fast()),
            Rule::op_target => Ok(Operator::Target()),
            Rule::op_degrade => Ok(Operator::Degrade()),
            Rule::op_replicate => Ok(Operator::Replicate()),
            _ => Err("unrecognized operator")
        }
    }
}

#[derive(Clone, Debug)]
struct Expression {
    operator: Operator,
    right: Box<Step>,
    left: Box<Step>,
}

#[derive(Clone, Debug)]
struct Bjorklund {
    left: Box<Step>,
    steps: Box<Step>,
    pulses: Box<Step>,
    rotation: Option<Box<Step>>,
}

#[derive(Clone, Debug, Default)]
enum StepValue {
    #[default] Rest,
    Hold,
    Float(f64),
    Integer(i32),
    Pitch(Pitch),
    Name(String),
}

impl StepValue {
    // parse a single into a value
    fn parse(pair: Pair<Rule>) -> StepValue {
        // println!("{:?}", pair);
        match pair.as_rule() {
            Rule::number => {
                if let Some(n) = pair.into_inner().next() {
                    match n.as_rule() {
                        Rule::integer => StepValue::Integer(n.as_str().parse::<i32>().unwrap_or(0)),
                        Rule::float => StepValue::Float(n.as_str().parse::<f64>().unwrap_or(0.0)),
                        Rule::normal => StepValue::Float(n.as_str().parse::<f64>().unwrap_or(0.0)),
                        _ => unreachable!(),
                    }
                } else {
                    unreachable!()
                }
            }
            Rule::hold => StepValue::Hold,
            Rule::rest => StepValue::Rest,
            Rule::pitch => StepValue::Pitch(Pitch::parse(pair)),
            Rule::name => StepValue::Name(pair.as_str().to_string()),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct Span {
    start: f64,
    end: f64,
}

impl Span {
    // transforms a nested relative span according to an absolute span at output time
    fn transform(&self, outer: &Span) -> Span {
        let start = outer.start + outer.length() * self.start;
        Span {
            start,
            end: start + outer.length() * self.length(),
        }
    }
    fn length(&self) -> f64 {
        self.end - self.start
    }
    fn default() -> Self {
        Span{
            start : 0.0,
            end : 1.0
        }
    }
    fn new(start: f64, end: f64) -> Self {
        Self { start, end }
    }
}

#[derive(Clone, Debug, Default)]
enum Target {
    #[default] None,
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
                    self
                    // self.span.start, self.span.end, self.value
                ))
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
struct SingleEvent {
    length: f64,
    span: Span,
    value: StepValue,
    target : Target, // value for instruments
}

impl Display for SingleEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{:.3} -> {:.3} | {:?} {}",
            self.span.start, self.span.end, self.value, self.target
            // self.span.start, self.span.end, self.value
        ))
    }
} 

#[derive(Debug, Clone)]
struct MultiEvents {
    length: f64,
    span: Span,
    events: Vec<Events>,
}

#[derive(Debug, Clone)]
struct PolyEvents {
    length: f64,
    span: Span,
    channels: Vec<Events>,
}

#[derive(Debug, Clone)]
enum Events {
    Single(SingleEvent),
    Multi(MultiEvents),
    Poly(PolyEvents),
}

impl Events {
    fn empty() -> Events {
        Events::Single(SingleEvent{
            length: 1.0,
            span: Span::default(),
            value: StepValue::Rest,
            target: Target::None
        })
    }
    // only applied for Subdivision and Polymeter groups
    fn subdivide_lengths(events: &mut Vec<Events>){
        let mut length = 0.0;
        for e in &mut *events {
            match e {
                Events::Single(s) => length += s.length,
                Events::Multi(m) => length += m.length,
                Events::Poly(p) => length += p.length,
            }
        }
        let step_size = 1.0 / (length as f64);
        let mut start = 0.0;
        for e in &mut *events {
            match e {
                Events::Single(s) => {
                    s.length *= step_size;
                    s.span  = Span::new(start, start + s.length);
                    start += s.length
                }
                Events::Multi(m) => {
                    m.length *= step_size;
                    m.span  = Span::new(start, start + m.length);
                    start += m.length
                }
                Events::Poly(p) => {
                    p.length *= step_size;
                    p.span  = Span::new(start, start + p.length);
                    start += p.length
                }
            }
        }
    }
    fn mutate_events<F>(&mut self, fun: &mut F)
    where
        F: FnMut(&mut SingleEvent),
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
    fn collect(&self, channels: &mut Vec<Vec<SingleEvent>>, channel: usize){
        if channels.len() <= channel {
            channels.push(vec![])
        }
        match self {
            Events::Single(s) => {
                channels[channel].push(s.clone())
            }
            Events::Multi(m) => {
                for e in &m.events {
                    e.collect(channels, channel);
                }
            }
            Events::Poly(p) => {
                let mut c = channel;
                for e in &p.channels {
                    e.collect(channels, c);
                    c += 1
                }
            }
        }
    }
}

// stacks can only appear inside groups like Subdivision, Alternating or Polymeter
// they will have a stack of steps with their parent's type inside
fn as_stack(pair: Pair<Rule>, parent: Pair<Rule>) -> Stack {
    let mut stack = Stack {
        stack: vec![],
    };

    match parent.as_rule() {
        Rule::alternating => {
            for p in pair.into_inner() {
                stack.stack.push(Step::Alternating(Alternating {
                    current: 0,
                    steps: section_as_steps(p),
                }))
            }
        }
        Rule::subdivision | Rule::mini => {
            for p in pair.clone().into_inner() {
                stack.stack.push(Step::Subdivision(Subdivision {
                    steps: section_as_steps(p),
                }))
            }
        }
        Rule::polymeter => {
            if let Some(count) = as_polymeter_count(&parent) {
                for p in pair.clone().into_inner() {
                    stack.stack.push(Step::Polymeter(Polymeter {
                        count,
                        offset: 0,
                        steps: section_as_steps(p),
                    }))
                }
            }
        }
        _ => (),
    }
    stack
}


fn as_polymeter_count(pair: &Pair<Rule>) -> Option<usize> {
    for p in pair.clone().into_inner() {
        match p.as_rule() {
            Rule::polymeter_tail => {
                // TODO a more generic parameter here
                if let Some(count) = p.into_inner().next() {
                    return Some(count.as_str().parse::<usize>().unwrap_or(1));
                }
            }
            _ => (),
        }
    }
    None
}

fn as_polymeter(pair: Pair<Rule>) -> Result<Step, &str> {
    if let Some(count) = as_polymeter_count(&pair) {
        let mut inner = pair.clone().into_inner();
        if let Some(poly_list) = inner.next() {
            return Ok(Step::Polymeter(Polymeter {
                count,
                offset: 0,
                steps: section_as_steps(poly_list),
            }));
        }
    }
    Err("invalid polymeter")
}

fn bjorklund_pattern(pulses: i32, steps: i32, rotation: Option<i32>) -> Vec<bool> {
    let slope = (pulses as f64) / (steps as f64);
    let mut pattern = vec![];
    let mut prev = -1.0;
    for i in 0..steps {
        let curr = ((i as f64) * slope).floor();
        pattern.push(curr != prev);
        prev = curr;
    }
    if let Some(rotate) = rotation {
        pattern.rotate_left(rotate as usize);
    }
    pattern
}

// recursively parse a pair as a Step
fn parse_step(pair: Pair<Rule>) -> Result<Step, &str> {
    match pair.as_rule() {
        Rule::single => {
            if let Some(v) = pair.into_inner().next() {
                Ok(Step::Single(Single {
                    string: v.as_str().to_string(),
                    value: StepValue::parse(v),
                }))
            } else {
                unreachable!()
            }
        }
        Rule::subdivision | Rule::mini => {
            if let Some(first) = pair.clone().into_inner().next() {
                match first.as_rule() {
                    Rule::stack => {
                        Ok(Step::Stack(as_stack(first, pair)))
                    }
                    _ => {
                        let sd = Subdivision {
                            steps: unwrap_section(pair),
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
                    Rule::stack => Ok(Step::Stack(as_stack(first, pair))),
                    _ => {
                        let a = Alternating {
                            current: 0,
                            steps: unwrap_section(pair),
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
                    Rule::stack => Ok(Step::Stack(as_stack(first, pair))),
                    Rule::polymeter_tail => Ok(Step::Single(Single::default())),
                    _ => as_polymeter(pair),
                }
            } else {
                Ok(Step::Single(Single::default()))
            }
        }
        Rule::stack | Rule::section | Rule::choices => {
            // stacks can only appear inside rules for Subdivision, Alternating or Polymeter
            // sections and choices are always immediately handled within other rules
            // using unwrap_section or section_as_steps
            Err("internal error, unexpected branch reached")
        }
        Rule::expr => {
            let mut inner = pair.clone().into_inner();
            if let Some(left_pair) = inner.next() {
                let left = parse_step(left_pair)?;
                if let Some(op) = inner.next() {
                    match op.as_rule() {
                        Rule::op_bjorklund => {
                            let mut op_inner = op.into_inner();
                            if let Some(pulse_pair) = op_inner.next() {
                                let pulses = parse_step(pulse_pair)?;
                                if let Some(steps_pair) = op_inner.next() {
                                    let steps = parse_step(steps_pair)?;
                                    let mut rotate = None;
                                    if let Some(rotate_pair) = op_inner.next() {
                                        rotate = Some(parse_step(rotate_pair)?);
                                    }
                                    return Ok(Step::Bjorklund(Bjorklund{
                                        left: Box::new(left),
                                        pulses: Box::new(pulses),
                                        steps: Box::new(steps),
                                        rotation: match rotate {
                                            Some(r) => Some(Box::new(r)),
                                            None => None
                                        }
                                    }))
                                }
                            }
                        }
                        _ => {
                            let operator = Operator::parse(op.clone())?;
                            if let Some(right_pair) = op.into_inner().next() {
                                let right = parse_step(right_pair)?;
                                let expr = Step::Expression(Expression {
                                    left: Box::new(left),
                                    right: Box::new(right),
                                    operator,
                                });
                                return Ok(expr)
                            }
                        }
                    }
                }
            }
            Err("incomplete expression")
        }
        _ => {
            Err("rule not implemented")
        }
    }
}

// helper to convert a section rule to a vector of Steps
fn section_as_steps(pair: Pair<Rule>) -> Vec<Step> {
    let mut steps = vec![];
    for pair in pair.into_inner() {
        match parse_step(pair) {
            Ok(s) => steps.push(s),
            Err(s) => println!("{:?}", s),
        }
    }
    steps
}

// helper to convert a section or single to a vector of Steps
fn unwrap_section(pair: Pair<Rule>) -> Vec<Step> {
    if let Some(inner) = pair.into_inner().next() {
        match inner.as_rule() {
            Rule::single => {
                if let Ok(s) = parse_step(inner) {
                    vec![s]
                } else {
                    vec![]
                }
            }
            Rule::section => {
                section_as_steps(inner)
            }
            Rule::choices => {
                let mut choices: Vec<Step> = vec![];
                for p in inner.into_inner() {
                    if let Some(step) = p.into_inner().next() {
                        if let Ok(choice) = parse_step(step) {
                            choices.push(choice)
                        }
                    }
                }
                vec![Step::Choices(Choices {
                    choices,
                })]
            }
            _ => {
                println!("{:?}", inner);
                unreachable!()
            }
        }
    } else {
        vec![]
    }
}

// recursively output events for the entire cycle based on some state (random seed)
fn output(step: &mut Step, state: State) -> Events {
    match step {
        Step::Single(s) => {
            Events::Single(SingleEvent {
                length: 1.0,
                target : Target::None,
                span: Span::default(),
                value: s.value.clone(),
            })
        }
        Step::Subdivision(sd) => {
            if sd.steps.len() == 0 {
                unreachable!()
            } else {
                let mut events = vec![];
                for s in &mut sd.steps {
                    let e = output(s, state.clone());
                    events.push(e)
                    // events.extend(output_events(s, state.clone()))
                }
                // only applied for Subdivision and Polymeter groups
                Events::subdivide_lengths(&mut events);
                Events::Multi(MultiEvents{
                    span: Span::default(),
                    length: 1.0,
                    events
                })
            }
        }
        Step::Alternating(a) => {
            if a.steps.len() == 0 {
                unreachable!()
            } else {
                let current = a.current % a.steps.len();
                a.current += 1;
                if let Some(step) = a.steps.get_mut(current) {
                    output(step, state)
                } else {
                    unreachable!()
                }
            }
        }
        Step::Choices(cs) => {
            // TODO move this outside
            let seed = state.seed.unwrap_or_else(|| thread_rng().gen());
            let mut rng = Xoshiro256PlusPlus::from_seed(seed);
            let choice = rng.gen_range(0..cs.choices.len());
            output(&mut cs.choices[choice], state)
        }
        Step::Polymeter(pm) => {
            if pm.steps.len() == 0 {
                unreachable!()
            } else {
                let mut events = vec![];
                let length = pm.steps.len();
                let offset = pm.offset;
                
                for i in 0..pm.count {
                    events.push(output(
                        &mut pm.steps[(offset + i) % length].clone(),
                        state.clone()
                    ))
                }
                pm.offset += pm.count;
                // only applied for Subdivision and Polymeter groups
                Events::subdivide_lengths(&mut events);
                Events::Multi(MultiEvents{
                    span: Span::default(),
                    length: 1.0,
                    events
                })
            }
        }
        Step::Stack(st) => {
            if st.stack.len() == 0 {
                unreachable!()
            } else {
                let mut channels = vec![];
                for s in &mut st.stack {
                    channels.push(output(s, state.clone()))
                }
                Events::Poly(PolyEvents{
                    span: Span::default(),
                    length:1.0,
                    channels
                })
            }
        }
        Step::Expression(e) => {
            match e.operator {
                Operator::Fast() => {
                    let mut events = vec![];
                    match e.right.as_ref() {
                        Step::Single(s) => {
                            if let Some(mult) = s.to_integer(){
                                for _i in 0..mult{
                                    events.push(output(&mut e.left, state.clone()))
                                }
                            }
                        }
                        _ => unreachable!()
                    }
                    Events::subdivide_lengths(&mut events);
                    Events::Multi(MultiEvents {
                        span: Span::default(),
                        length: 1.0,
                        events,
                    })
                }
                Operator::Target() => {
                    let mut out = output(e.left.as_mut(), state.clone());
                    match e.right.as_ref() {
                        Step::Single(s) => {
                            out.mutate_events(&mut |e| 
                                e.target = s.to_target()
                            )
                        }
                        _ => unreachable!()
                    }
                    out
                }
                Operator::Degrade() => {
                    let mut out = output(e.left.as_mut(), state.clone());
                    match e.right.as_ref() {
                        Step::Single(s) => {
                            out.mutate_events(&mut |e : &mut SingleEvent|
                                if let Some(chance) = s.to_chance(){
                                    // TODO move this outside
                                    let seed = state.seed.unwrap_or_else(|| thread_rng().gen());
                                    let mut rng = Xoshiro256PlusPlus::from_seed(seed);
                                    let random = rng.gen_range(0.0..1.0);
                                    if chance < random {
                                        e.value = StepValue::Rest
                                    }
                                }
                            )
                        }
                        _ => unreachable!()
                    }
                    out
                }
                Operator::Replicate() => {
                    let mut events = vec![];
                    match e.right.as_ref() {
                        Step::Single(s) => {
                            if let Some(mult) = s.to_integer(){
                                let out = output(&mut e.left, state.clone());
                                for _i in 0..mult{
                                    events.push(out.clone())
                                }
                            }
                        }
                        _ => unreachable!()
                    }
                    Events::subdivide_lengths(&mut events);
                    Events::Multi(MultiEvents {
                        span: Span::default(),
                        length: 1.0,
                        events,
                    })
                }
            }
        }
        Step::Bjorklund(b) => {
            let mut events = vec![];
            match b.pulses.as_ref() {
                Step::Single(pulses_single) => {
                    match b.steps.as_ref(){
                        Step::Single(steps_single) => {
                            let rotation = match &b.rotation {
                                Some(r) => match r.as_ref() {
                                    Step::Single(rotation_single) => {
                                        if let Some(r) = rotation_single.to_integer() {
                                            Some(r)
                                        }else{
                                            None
                                        }
                                    }
                                    _ => unreachable!()
                                }
                                None => None
                            };
                            if let Some(pulses) = pulses_single.to_integer() {
                                if let Some(steps) = steps_single.to_integer(){
                                    let out = output(&mut b.left, state.clone());
                                    for pulse in bjorklund_pattern(pulses, steps, rotation) {
                                        if pulse {
                                            events.push(out.clone())
                                        }else{
                                            events.push(Events::empty())
                                        }
                                    }
                                }
                            }
                        }
                        _ => unreachable!()
                    }
                }
                _ => unreachable!()
            }
            Events::subdivide_lengths(&mut events);
            Events::Multi(MultiEvents {
                span: Span::default(),
                length: 1.0,
                events,
            })
        }
        // _ => Events::Single(SingleEvent::default())
    }
}

fn crawl_step(step: &mut Step, fun: fn(&mut Step, usize), level: usize) {
    fun(step, level);
    match step {
        Step::Single(_s) => (),
        Step::Alternating(a) => {
            for s in &mut a.steps {
                crawl_step(s, fun, level + 1)
            }
        }
        Step::Polymeter(pm) => {
            for s in &mut pm.steps {
                crawl_step(s, fun, level + 1)
            }
        }
        Step::Subdivision(sd) => {
            for s in &mut sd.steps {
                crawl_step(s, fun, level + 1)
            }
        }
        Step::Choices(cs) => {
            for s in &mut cs.choices {
                crawl_step(s, fun, level + 1)
            }
        }
        Step::Stack(st) => {
            for s in &mut st.stack {
                crawl_step(s, fun, level + 1)
            }
        }
        Step::Expression(e) => {
            crawl_step(e.left.as_mut(), fun, level + 1);
            crawl_step(e.right.as_mut(), fun, level + 1)
        }
        Step::Bjorklund(b) => {
            crawl_step(b.left.as_mut(), fun, level + 1);
            crawl_step(b.steps.as_mut(), fun, level + 1);
            crawl_step(b.pulses.as_mut(), fun, level + 1);
            if let Some(rotation) = b.rotation.as_mut() {
                crawl_step(rotation, fun, level + 1);
            }
        }
    }
}

fn reset_step(step: &mut Step, _level: usize) {
    match step {
        Step::Alternating(a) => {
            a.current = 0;
        }
        Step::Polymeter(pm) => {
            pm.offset = 0;
        }
        _ => (),
    }
}

// parse the root pair of the pest AST into a Subdivision
// then update the spans of all the generated steps
fn parse_tree(tree: &Pair<Rule>) -> Result<Step, String> {
    let cycle = parse_step(tree.clone())?;
    // update_span(&mut cycle, &Span::default());
    Ok(cycle)
}

// parse a string into a step
fn parse(input: &str) -> Result<Step, String> {
    match MiniParser::parse(Rule::mini, &input) {
        Ok(mut tree) => parse_tree(&tree.next().unwrap()),
        Err(err) => Err(format!("{}", err)),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn indent_lines(level: usize) -> String {
        let mut lines = String::new();
        for i in 0..level {
            lines += [" │", " |"][i % 2];
        }
        lines
    }

    fn print_steps(step: &mut Step, level: usize) {
        let name = match step {
            Step::Single(s) => match &s.value {
                StepValue::Pitch(_p) => format!("{:?} {}", s.value, s.string),
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
        println!("{} {}", indent_lines(level), name);
    }

    fn print_pairs(pair: &Pair<Rule>, level: usize) {
        println!(
            "{} {:?} {:?}",
            indent_lines(level),
            pair.as_rule(),
            pair.as_str()
        );
        for p in pair.clone().into_inner() {
            print_pairs(&p, level + 1)
        }
    }

    fn set_event_spans(events: &mut Events, span: &Span) {
        let unit = span.length();
        match events {
            Events::Single(s) => {
                s.length *= unit;
                s.span = s.span.transform(&span);
            }
            Events::Multi(m) => {
                m.length *= unit;
                m.span = m.span.transform(&span);

                for e in &mut m.events {
                    set_event_spans(e, &m.span);
                }
            }
            Events::Poly(p) => {
                p.length *= unit;
                p.span = p.span.transform(&span);
                for e in &mut p.channels {
                    set_event_spans(e, &p.span);
                }
            }
        }
    }

    fn parse_with_debug(input: &str) {
        println!("\n{}", "=".repeat(42));
        println!("\nINPUT\n {:?}", input);

        match MiniParser::parse(Rule::mini, &input) {
            Ok(mut tree) => {
                let mini = tree.next().unwrap();
                println!("\nTREE");
                print_pairs(&mini, 0);
                match parse_tree(&mini) {
                    Ok(mut step) => {
                        println!("\nCYCLE");
                        crawl_step(&mut step, print_steps, 0);
                        let stateful_chars = ['<', '{', '|', '?'];
                        let repeats = if stateful_chars.iter().any(|&c| input.contains(c)) {
                            5
                        } else {
                            1
                        };
                        println!("\nOUTPUT");
                        for i in 0..repeats {
                            println!(" {}", i);
                            let mut events = output(&mut step, State { seed: None });
                            set_event_spans(&mut events, &Span::default());
                            // println!("{:#?}", events);

                            let mut channels = vec![];
                            events.collect(&mut channels, 0);

                            let mut ci = 0;
                            let channel_count = channels.len();
                            for channel in &mut channels {
                                if channel_count > 1 {
                                    println!(" /{}", ci);
                                }
                                let mut i = 0;
                                for event in channel {
                                    println!("  │{:02}│ {}", i, event);
                                    i += 1
                                }
                                ci += 1

                            }
                            // crawl_step(&mut step, reset_step, 0);
                        }
                    }
                    Err(s) => println!("{}", s),
                }
            }
            Err(err) => println!("{}", err),
        }
    }

    #[test]
    pub fn test_tidal() {
        // let file = fs::read_to_string("rhythm/test.tidal").expect("file not found");
        // let line = file.lines().next().expect("no lines in file");
        // println!("\nInput:\n{:?}\n", line);

        parse_with_debug("a b c d");
        parse_with_debug("a b [c d]");
        parse_with_debug("[a a] [b b b] [c d c d]");
        parse_with_debug("[a [b [c d]]] e [[[f g] a ] b]");
        parse_with_debug("[a [b [c d]]] , [[[f g] a ] b]");
        parse_with_debug("a b <c d>");
        parse_with_debug("<a b , <c d>>");
        parse_with_debug("<a <b c d> e <f <a b c d>> <g a b>>");
        parse_with_debug("{a b c d e}%4");
        parse_with_debug("{a b c d e}%3");
        parse_with_debug("{a b , c d e f g}%3");
        parse_with_debug("{a b <c d e f> [g a]}%3");
        parse_with_debug("[1 2 ~] {}%42 [] <>");
        parse_with_debug("1 [2 [3 [4 [5 [6 [7 [8 [9 10]]]]]]]]");
        parse_with_debug("1 [2 <[3 4] <5 [6 7] [6 _ 7] [8 9 10]>>]");
        parse_with_debug("[1 2] [3 4, 5 6]");
        parse_with_debug("a*2 b c");
        parse_with_debug("a b c [d e]*4");
        parse_with_debug("a:1 b:2 c:3 [d e f g]:4");
        parse_with_debug("[a b c d]?0.5");
        parse_with_debug("a b!2 c!3 d!4 [1 2 3 4]!5");
        parse_with_debug("a(6,8)");
        parse_with_debug("[a b](3,8)");
    }
}
