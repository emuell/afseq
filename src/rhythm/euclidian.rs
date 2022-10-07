/// Generates an euclidian rhythm pattern with the given pulse and step count
pub fn euclidean(pulses: u32, steps: u32) -> Vec<bool> {
    type Group = Vec<bool>;
    type Groups = Vec<Group>;

    fn generate(mut fgs: Groups, mut lgs: Groups) -> Groups {
        if lgs.len() < 2 {
            fgs.append(&mut lgs.clone());
        } else {
            let mut nfgs: Groups = Vec::new();
            while !fgs.is_empty() && !lgs.is_empty() {
                let mut flg = fgs.last().unwrap().clone();
                let mut llg = lgs.last().unwrap().clone();
                flg.append(&mut llg);

                nfgs.push(flg);
                fgs.pop();
                lgs.pop();
            }
            fgs.append(&mut lgs);
            return generate(nfgs, fgs);
        }
        fgs
    }

    if pulses < steps {
        let mut rhythm: Group = Vec::with_capacity(steps as usize);

        let front = vec![vec![true]; pulses as usize];
        let last = vec![vec![false]; (steps - pulses) as usize];

        let rhythms = generate(front, last);
        for g in rhythms {
            for i in g {
                rhythm.push(i);
            }
        }
        rhythm
    } else {
        vec![true; steps as usize]
    }
}
