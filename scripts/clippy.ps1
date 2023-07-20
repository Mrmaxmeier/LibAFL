cargo clippy --all --all-features --tests --benches --examples -- `
   -D clippy::all `
   -D clippy::pedantic `
   -W clippy::similar_names `
   -A clippy::type_repetition_in_bounds `
   -A clippy::missing-errors-doc `
   -A clippy::cast-possible-truncation `
   -A clippy::used-underscore-binding `
   -A clippy::ptr-as-ptr `
   -A clippy::missing-panics-doc `
   -A clippy::missing-docs-in-private-items `
   -A clippy::unseparated-literal-suffix `
   -A clippy::module-name-repetitions `
   -A clippy::unreadable-literal

cd libafl_libfuzzer\libafl_libfuzzer_runtime
cargo clippy --all --all-features --tests --benches --examples -- `
   -D clippy::all `
   -D clippy::pedantic `
   -W clippy::similar_names `
   -A clippy::type_repetition_in_bounds `
   -A clippy::missing-errors-doc `
   -A clippy::cast-possible-truncation `
   -A clippy::used-underscore-binding `
   -A clippy::ptr-as-ptr `
   -A clippy::missing-panics-doc `
   -A clippy::missing-docs-in-private-items `
   -A clippy::unseparated-literal-suffix `
   -A clippy::module-name-repetitions `
   -A clippy::unreadable-literal
