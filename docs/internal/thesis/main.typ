#import "/packages.typ": *
#import "lib.typ": *
#import packages.codly: *
#import packages.codly-languages: *

#set text(font: "Liberation Serif", lang: "fr", hyphenate: false, size: 12pt)
#set par(
    justify: true,
    first-line-indent: 1cm,
    leading: 1em,
    spacing: 1.75em,
)

#set page(
    numbering: "I",
    // margin: (left: 5cm, right: 5cm),
    // width: 210.0mm + 5cm,
    // height: 297.0mm,
)

#show std.figure: set std.figure(supplement: "Figure")
#show std.figure.where(kind: table): set std.figure(supplement: "Tableau")

#show link: set text(blue)
#show link: underline
#show: codly-init.with()
#codly(languages: codly-languages)

#outline()
#pagebreak(weak: true)
#outline-figure()
#pagebreak(weak: true)
#todo[
    - Acronyms list
    - Table of appendices
]

#pagebreak(weak: true)

#todo[Remerciements]
// #include "extra/acknowledgements.typ"

#include "/subject-statement/main.typ"

#todo[Résumé]
// #include "/abstract/main.typ"

// = Utilisation de l'intelligence artificielle
// = Abréviations, termes et définitions
// = Conventions utilisées dans le document


#set page(numbering: "1/1")
#counter(page).update(1)
#include "contents/introduction.typ"

#set heading(numbering: "1.")
#show heading.where(level: 1): it => {
    pagebreak(weak: true)
    it
}

#include "contents/functional-overview.typ"
#include "contents/system-design.typ"
#include "contents/implementation.typ"
#include "contents/validation.typ"
#include "contents/comparison.typ"
#include "contents/results.typ"
#include "contents/discussion.typ"

#set heading(numbering: none)
#include "contents/conclusion.typ"

#bibliography("../bibliography.yaml")
