use std::collections::{LinkedList, HashMap};
use std::ops::RangeInclusive;

#[derive(Debug)]
struct Production {
    nonterminal: char,
    alternative: String
}

impl Production {
    fn new(nonterminal: char, alternative: &str) -> Production {
        Production {
            nonterminal,
            alternative: alternative.to_string(),
        }
    }
    
    fn p_bar(&self) -> usize {
        self.alternative.len()
    }
}

#[derive(Debug)]
struct Grammar {
    productions: Vec<Production>,
}

impl Grammar {
    fn new(mut productions: Vec<Production>, root: char) -> Grammar {
        let mut prod = vec![
            Production {
                nonterminal: 'F', // TODO fix this maybe
                alternative: root.to_string()+"\0"
            }];
        prod.append(&mut productions);

        Grammar {
            productions: prod,
        }
    }
    
    fn index_production(&self, p: usize, j: usize, alpha: &str) -> char {
        assert!(p < self.productions.len());
        if j >= self.productions[p].alternative.len() {
            let after = j - self.productions[p].alternative.len();
            return char::from(alpha.as_bytes()[after]);
        }
        char::from(self.productions[p].alternative.as_bytes()[j])
    }
    
    fn is_nonterminal(&self, c: char) -> bool {
        c.is_uppercase()
    }
    
    fn is_terminal(&self, c: char) -> bool {
        !self.is_nonterminal(c)
    }
    
    /// Data structure 1
    fn by_nonterminal(&self, nonterminal: char) -> LinkedList<String> {
        assert!(nonterminal.is_uppercase());
        
        self.productions.iter().filter(|x| x.nonterminal == nonterminal).map(|x| x.alternative.clone()).collect()
    }
    
    fn is_nonfinal(&self, state: &State) -> bool {
        state.j != self.productions[state.p].p_bar()
    }
    
    fn is_final(&self, state: &State) -> bool {
        !self.is_nonfinal(state)
    }
}

struct Recognizer {
    grammar: Grammar,
    lookahead: usize,
}

impl Recognizer {
    fn new(grammar: Grammar, lookahead: usize) -> Recognizer {
        Recognizer {
            grammar,
            lookahead,
        }
    }
    
    fn slice_with_lookahead(&self, input: &str, index: usize) -> String {
        let end = index + self.lookahead;
        // needs to pad with null when sliced past end
        if end <= input.len() {
            input[index..end].to_string()
        }
        else {
            let diff = end - input.len();
            let mut result = input[index..].to_string();
            result.push_str(&"\0".repeat(diff));
            result
        }
    }
    
    fn recognize(&self, input: &str) -> bool {
        let input_padded = input.to_string() + &"\0".repeat(self.lookahead+1);
        // State sets are linked lists, data structure 2
        let mut state_sets = Vec::<LinkedList::<State>>::with_capacity(input.len());
        state_sets.resize_with(input.len()+1+self.lookahead, Default::default);
        // Data structure 4, for completer
        // Vec == "for each state set Si"
        // HashMap == "and nonterminal N"
        // LinkedList == "a list of all states such that..."
        let mut completer_ds = Vec::<HashMap::<char, LinkedList<State>>>::with_capacity(input.len());
        completer_ds.resize_with(input.len(), Default::default);
        for hashmap in completer_ds.iter_mut() {
            for production in self.grammar.productions.iter() {
                hashmap.insert(production.nonterminal, LinkedList::new());
            }
        }
        
        state_sets[0].push_back(State {
            p: 0,
            j: 0,
            f: 0,
            alpha: "\0".repeat(self.lookahead),
        });
        // don't think i need to pre-pop anything here for the completer?

        dbg!(&self.grammar);
        
        for i in 0..input.len()+1 {
            println!("starting character loop {}", i);
            if !(state_sets.len() > i) {
                println!("early out");
                return false;
            }
            
            // Data structure 3
            let mut ds3 = Vec::<LinkedList::<State>>::with_capacity(i);
            ds3.resize_with(i+1, Default::default);
            
            while let Some(s) = state_sets[i].pop_back() {
                
                let c_p_j_1 = self.grammar.index_production(s.p, s.j, "\0");
                println!("state: {:?}, index: {:?}, cpj1: {:?}.", s, i, c_p_j_1);
                println!("final?: {:?}, terminal?: {:?}", self.grammar.is_final(&s),self.grammar.is_terminal(c_p_j_1));
                // predictor
                if self.grammar.is_nonfinal(&s) && self.grammar.is_nonterminal(c_p_j_1) {
                    completer_ds[i].get_mut(&c_p_j_1).unwrap().push_back(s.clone());
                    for (q, production) in self.grammar.productions.iter().enumerate().filter(|(_, x)| x.nonterminal == c_p_j_1)  {
                        let state = State {
                            p: q,
                            j: 0,
                            f: i,
                            alpha: self.grammar.index_production(s.p, s.j+1, &s.alpha).to_string(), // TODO non-1
                            //alpha: self.slice_with_lookahead(&production.alternative, s.j+1),
                        };
                        if !ds3[state.f].contains(&state) {
                            println!("Adding state {:?} to set {}", state, i);
                            ds3[state.f].push_back(state.clone());
                            state_sets[i].push_back(state.clone());
                        }
                    }
                }
                
                // completer
                if self.grammar.is_final(&s) && input_padded[i..=i+self.lookahead].contains(&s.alpha) {
                    let nonterminal = self.grammar.productions[s.p].nonterminal;
                    for fstate in completer_ds[s.f][&nonterminal].iter() {
                        let state = State {
                            p: fstate.p,
                            j: fstate.j+1,
                            f: fstate.f,
                            alpha: fstate.alpha.clone()
                        };
                        if !state_sets[i].contains(&state) {
                            println!("Adding state {:?} to set {}", state, i);
                            state_sets[i].push_back(state.clone());
                        }
                    }
                }
                
                // scanner
                if self.grammar.is_nonfinal(&s) && self.grammar.is_terminal(c_p_j_1) {
                    let x_i_1 = input_padded.chars().nth(i).unwrap();
                    println!("Xi+1 == {:?}", x_i_1);
                    if c_p_j_1 == x_i_1 {
                        let state = State {
                            p: s.p,
                            j: s.j+1,
                            f: s.f,
                            alpha: s.alpha.clone(),
                        };
                        // TODO this isn't strictly how the paper describes this
                        if !state_sets[i+1].contains(&state) {
                            println!("Adding state {:?} to set {}", state, i+1);
                            state_sets[i+1].push_back(state);
                        }
                    }
                }
            }
        }
        state_sets[input.len()+1].contains(&State {
            p: 0,
            j: 2,
            f: 0,
            alpha: "\0".to_string(),
        })
    }
}

#[derive(PartialEq, Clone, Debug)]
struct State {
    p: usize,
    j: usize,
    f: usize,
    alpha: String,
}

impl State {
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let p1 = Production::new('E', "T");
        let p2 = Production::new('E', "E+T");
        let p3 = Production::new('T', "P");
        let p4 = Production::new('T', "T*P");
        let p5 = Production::new('P', "a");
        
        let g = Grammar::new(vec![p1, p2, p3, p4, p5], 'E');
        
        let r = Recognizer::new(g, 1);
        
        assert!(r.recognize("a+a*a"));
    }
}
