#import "lib.typ": *

#import "../lib.typ": *
#import "../../packages.typ": *
#import packages.lilaq as lq

#let (memory,) = lq.load-txt(read("membench.csv"))
