pub mod alphabet;
pub mod data;
pub mod dfa;
pub mod nfa;
pub mod pattern;
pub mod state;
pub mod symbol;

pub use dfa::*;
pub use nfa::*;
pub use pattern::*;
pub use symbol::*;

use enso_prelude as prelude;


use prelude::*;

pub fn main() {
    let mut nfa : NFA = default();
    let start = nfa.new_state();
    let end_a   = nfa.new_pattern(start,&Pattern::char('a'));
    let end_b   = nfa.new_pattern(end_a,&Pattern::char('b'));
    let end_c   = nfa.new_pattern(end_b,&Pattern::char('c'));
    let end_x   = nfa.new_pattern(start,&Pattern::char('x'));

    nfa.states[end_a.id].name = Some("end_a".into());
    nfa.states[end_c.id].name = Some("end_c".into());
    nfa.states[end_x.id].name = Some("end_x".into());

    let dfa = DFA::from(&nfa);

    println!("start: {:?}",start);
    println!("end_a: {:?}",end_a);
    println!("end_b: {:?}",end_b);
    println!("end_c: {:?}",end_c);
    println!("end_x: {:?}",end_x);
    println!("{:#?}",dfa);

    let t = &dfa.alphabet_segmentation;

    println!("---------");
    let after_a = dfa.next_state(state::Identifier::from(0),Symbol::new(97));
    let after_b = dfa.next_state(after_a,Symbol::new(98));
    let after_c = dfa.next_state(after_b,Symbol::new(99));
    println!("{:?}",after_a);
    println!("{:?}",after_b);
    println!("{:?}",after_c);


}


fn get(map:&BTreeMap<Symbol,usize>, symbol:Symbol) -> Option<usize> {
    map.range(symbol..).next().map(|(k,v)|{
        if *k == symbol { *v } else { v - 1 }
    })
}