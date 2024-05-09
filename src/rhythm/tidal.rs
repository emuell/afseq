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
    note: i16,
    octave: u8,
}

impl Pitch {
    fn as_note_value(note: char) -> Option<i16> {
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
            note: 60,
            octave: 4,
        };
        for p in pair.into_inner() {
            match p.as_rule() {
                Rule::note => {
                    if let Some(c) = String::from(p.as_str()).to_ascii_lowercase().chars().next() {
                        pitch.note = Pitch::as_note_value(c).unwrap_or(pitch.note)
                    }
                }
                Rule::octave => pitch.octave = p.as_str().parse::<u8>().unwrap_or(pitch.octave),
                Rule::mark => match p.as_str() {
                    "#" => pitch.note += 1,
                    "b" => pitch.note -= 1,
                    _ => (),
                },
                _ => (),
            }
        }
        // maybe an error should be thrown instead of a silent clamp
        pitch.note = pitch.note.clamp(0, 127);
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
}

#[derive(Clone, Debug, Default)]
struct Single {
    value: StepValue,
    target : Option<i16>, // value for instruments
    string: String,
}

impl Single {
    fn default() -> Self {
        Single {
            value: StepValue::Rest,
            target: None,
            string: String::from("~")
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

#[derive(Debug)]
struct CycleEvent {
    span: Span,
    length : f64,
    value: StepValue,
}

impl Display for CycleEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{:.3} -> {:.3} | {:?}",
            self.span.start, self.span.end, self.value
        ))
    }
} 

#[derive(Debug, Clone)]
struct SingleEvent {
    length: f64,
    span: Span,
    value: StepValue,
}

impl Display for SingleEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{:.3} -> {:.3} | {:?}",
            self.span.start, self.span.end, self.value
            // self.span.start, self.span.end, self.value
        ))
    }
} 

#[derive(Debug)]
struct MultiEvents {
    length: f64,
    span: Span,
    events: Vec<Events>,
}

#[derive(Debug)]
struct PolyEvents {
    length: f64,
    span: Span,
    channels: Vec<Events>,
}

#[derive(Debug)]
enum Events {
    Single(SingleEvent),
    Multi(MultiEvents),
    Poly(PolyEvents),
}

impl Events {
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

// recursively parse a pair as a Step
fn as_step(pair: Pair<Rule>) -> Result<Step, &str> {
    match pair.as_rule() {
        Rule::single => {
            if let Some(v) = pair.into_inner().next() {
                Ok(Step::Single(Single {
                    string: v.as_str().to_string(),
                    value: StepValue::parse(v),
                    target : None,
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
        _ => Err("type not implemented"),
    }
}

// helper to convert a section rule to a vector of Steps
fn section_as_steps(pair: Pair<Rule>) -> Vec<Step> {
    let mut steps = vec![];
    for pair in pair.into_inner() {
        match as_step(pair) {
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
                if let Ok(s) = as_step(inner) {
                    vec![s]
                } else {
                    vec![]
                }
            }
            Rule::section => section_as_steps(inner),
            Rule::choices => {
                let mut choices: Vec<Step> = vec![];
                for p in inner.into_inner() {
                    if let Some(step) = p.into_inner().next() {
                        if let Ok(choice) = as_step(step) {
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
                length: match s.value {
                    StepValue::Hold => 2.0,
                    _ => 1.0
                },
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
        // _ => Events::Single(SingleEvent {length : 1.0, span: Span::default(), value:StepValue::Rest})
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
    let mut cycle = as_step(tree.clone())?;
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
                        let stateful_chars = ['<', '{', '|'];
                        let repeats = if stateful_chars.iter().any(|&c| input.contains(c)) {
                            8
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
        // parse_with_debug("1 2, 3 4");
        
    }
}
