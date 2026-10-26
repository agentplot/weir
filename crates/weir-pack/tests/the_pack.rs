//! weir's own pack passes its own check, held to "weir's pack passes its
//! own check" (conformance-pack; 67; core.35).

use std::collections::BTreeSet;

use weir_pack::{check, satisfies, Clause, Fault, Pack, Shape};

#[test]
fn the_repositorys_pack() {
    let pack = Pack::embedded();
    let checked = check(&pack).unwrap_or_else(|faults| {
        panic!(
            "{}",
            faults
                .iter()
                .map(Fault::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        )
    });
    for shape in Shape::ALL {
        assert!(checked.read[&shape] > 0, "no case of {shape} read");
    }
    println!("{checked}");
}

#[test]
fn the_branch_cases_cite_44_to_50() {
    let pack = Pack::embedded();
    let cited: BTreeSet<Clause> = pack
        .cases(Shape::Branch)
        .iter()
        .flat_map(|case| satisfies(case).unwrap_or_else(|why| panic!("{why}")))
        .collect();
    for clause in 44..=50 {
        assert!(
            cited.contains(&Clause::number(clause)),
            "no branch case cites {clause}"
        );
    }
}
