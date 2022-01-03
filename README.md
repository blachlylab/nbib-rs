# nbib

MEDLINE/Pubmed .nbib format to CSL-JSON conversion 

## Status

Unstable and WIP

(but all components functional except for `reduce_authors`)

## Panics

There are some residual `unwrap()` calls scattered throughtout that I'll work to remove.

No unsafe code.

## Caveats

Needs nightly rust for feature gate `#![feature(slice_group_by)]`