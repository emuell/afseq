// -------------------------------------------------------------------------------------------------

/// Generates a Euclidean rhythm with the given number of steps, pulses, and rotation offset.
pub fn euclidean(steps: u32, pulses: u32, offset: i32) -> Vec<bool> {
    type Pattern = Vec<bool>;
    type Patterns = Vec<Pattern>;

    /// Recursively combines front and last groups to generate the Euclidean rhythm pattern.
    fn combine_groups(mut front_groups: Patterns, mut last_groups: Patterns) -> Patterns {
        if last_groups.len() < 2 {
            front_groups.append(&mut last_groups.clone());
        } else {
            let mut new_front_groups: Patterns =
                Vec::with_capacity(front_groups.len() + last_groups.len());
            while !front_groups.is_empty() && !last_groups.is_empty() {
                let mut front_last_group = front_groups.pop().unwrap();
                let mut last_last_group = last_groups.pop().unwrap();
                front_last_group.append(&mut last_last_group);

                debug_assert!(new_front_groups.capacity() > new_front_groups.len());
                new_front_groups.push(front_last_group);
            }
            front_groups.append(&mut last_groups);
            return combine_groups(new_front_groups, front_groups);
        }
        front_groups
    }

    if steps == 0 {
        vec![false; pulses as usize]
    } else if steps >= pulses {
        vec![true; pulses as usize]
    } else {
        let mut rhythm: Pattern = Vec::with_capacity(pulses as usize);

        let front_groups = vec![vec![true]; steps as usize];
        let last_groups = vec![vec![false]; (pulses - steps) as usize];

        let combined_patterns = combine_groups(front_groups, last_groups);
        for group in combined_patterns {
            for value in group {
                rhythm.push(value);
            }
        }

        match offset {
            n if n > 0 => rhythm.rotate_left((n as usize) % (pulses as usize)),
            n if n < 0 => rhythm.rotate_right((-n as usize) % (pulses as usize)),
            _ => (),
        }

        rhythm
    }
}

// --------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::euclidean;

    #[test]
    fn patterns() {
        // patterns from Toussaint
        let check_pattern = |(steps, pulses), result: &str| {
            let result = result.split(' ').map(|c| c == "x").collect::<Vec<bool>>();
            assert_eq!(euclidean(steps, pulses, 0), result);
        };
        check_pattern((1, 2), "x ~");
        check_pattern((1, 3), "x ~ ~");
        check_pattern((1, 4), "x ~ ~ ~");
        check_pattern((4, 12), "x ~ ~ x ~ ~ x ~ ~ x ~ ~");
        check_pattern((2, 5), "x ~ x ~ ~");
        check_pattern((3, 4), "x x x ~");
        check_pattern((3, 5), "x ~ x ~ x");
        check_pattern((3, 7), "x ~ x ~ x ~ ~");
        check_pattern((3, 8), "x ~ ~ x ~ ~ x ~");
        check_pattern((4, 7), "x ~ x ~ x ~ x");
        check_pattern((4, 9), "x ~ x ~ x ~ x ~ ~");
        check_pattern((4, 11), "x ~ ~ x ~ ~ x ~ ~ x ~");
        check_pattern((5, 6), "x x x x x ~");
        check_pattern((5, 7), "x ~ x x ~ x x");
        check_pattern((5, 8), "x ~ x x ~ x x ~");
        check_pattern((5, 9), "x ~ x ~ x ~ x ~ x");
        check_pattern((5, 11), "x ~ x ~ x ~ x ~ x ~ ~");
        check_pattern((5, 12), "x ~ ~ x ~ x ~ ~ x ~ x ~");
        check_pattern((5, 16), "x ~ ~ x ~ ~ x ~ ~ x ~ ~ x ~ ~ ~");
        check_pattern((7, 8), "x x x x x x x ~");
        check_pattern((7, 12), "x ~ x x ~ x ~ x x ~ x ~");
        check_pattern((7, 16), "x ~ ~ x ~ x ~ x ~ ~ x ~ x ~ x ~");
        check_pattern((9, 16), "x ~ x x ~ x ~ x ~ x x ~ x ~ x ~");
        check_pattern((11, 24), "x ~ ~ x ~ x ~ x ~ x ~ x ~ ~ x ~ x ~ x ~ x ~ x ~");
        check_pattern((13, 24), "x ~ x x ~ x ~ x ~ x ~ x ~ x x ~ x ~ x ~ x ~ x ~");
        // steps > pulses
        assert_eq!(
            euclidean(9, 8, 0),
            [true, true, true, true, true, true, true, true]
        );
        // empty steps
        assert_eq!(euclidean(0, 8, 0), vec![false; 8]);
        // empty pulses
        assert_eq!(euclidean(8, 0, 0), Vec::<bool>::new());
        // rotate
        assert_eq!(
            euclidean(3, 8, 3),
            [true, false, false, true, false, true, false, false]
        );
        assert_eq!(
            euclidean(3, 8, -3),
            [false, true, false, true, false, false, true, false]
        );
        // rotate and wrap
        assert_eq!(euclidean(3, 8, 5), euclidean(3, 8, 5 + 8));
        assert_eq!(euclidean(3, 8, -3), euclidean(3, 8, -3 - 8));
    }
}
