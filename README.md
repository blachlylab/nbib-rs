# nbib

MEDLINE/Pubmed .nbib format to CSL-JSON conversion 

## Dependencies

None ✅

## Status

API should be considered unstable, but all individual components are functional

`transmogrify` as an end-to-end conversion pipeline is WIP.

## Panics

There are some residual `unwrap()` calls scattered throughtout that I'll work to remove.

No unsafe code. ✅

## Caveats

?

(No longer needs nightly rust for feature gate `#![feature(slice_group_by)]`)
