#import "/packages.typ": *
#import "/lib/templates/cover/lib.typ": *
#import "lib.typ": *
#import "/lib/_utils.typ"
#import packages.codly: *
#import packages.codly-languages: *

#set document(
    author: "ARROYO Frédéric",
    title: "OS pour le déploiement de services conteneurisés",
    date: datetime(year: 2026, month: 09, day: 01),
    keywords: ("conteneurs", "système d'exploitation"),
)
#show: _utils.common-config

#cover(
    title: [OS pour le déploiement de services conteneurisés],
    submission: (
        statement: [Thèse de Bachelor présentée par],
        author: [ARROYO Frédéric],
        date: [Septembre 2026],
    ),
    degree-statement: [pour l'obtention du titre de Bachelor of Science HES-SO
        en],
    field-of-study: [
        Informatique et systèmes de communication avec orientation en \
        Informatique logicielle
    ],
    supervisors: (
        statement: [Professeur HES responsable],
        names: [
            GLÜCK Florent
        ],
    ),

    illustration: (
        image: image("/lib/assets/containeros.jpg"),
        legend-statement: [Légende et source de l'illustration de couverture:],
        legend: [
            La mascotte de Linux à côté d'un conteneur cargo, Swapneel MEHTA
            pour OpenSourceForU.com

        ],
    ),
    client: none,
)

#set par(
    justify: true,
    first-line-indent: 1cm,
    leading: 1em,
    spacing: 1.75em,
)

#set page(
    header: [
        #set text(size: 8pt)
        Thèse de Bachelor #sym.dash Septembre 2026
        #h(1fr)
        OS pour le déploiement de services conteneurisés
    ],
    footer: context [
        #set text(size: 10pt)

        ARROYO Frédéric
        #h(1fr)
        *#counter(page).display("I")*
    ],
)

#show std.figure: set std.figure(supplement: "Figure")
#show std.figure.where(kind: table): set std.figure(supplement: "Tableau")

#pagebreak(weak: true)
#full-outline()
#pagebreak(weak: true)

#show link: set text(blue)
#show link: underline
#show: codly-init.with()
#codly(languages: codly-languages)

#include "extra/acknowledgements.typ"
#{
    set page(
        header: none,
        footer: context [
            #h(1fr)
            *#counter(page).display("I")*
        ],
    )

    include "/subject-statement/main.typ"
    include "/abstract/main.typ"
}

#include "glossary.typ"

#set page(
    footer: context [
        #set text(size: 10pt)

        ARROYO Frédéric
        #h(1fr)
        *#counter(page).display("1")*
    ],
)
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
#include "contents/results-and-discussion.typ"

#set heading(numbering: none)
#include "contents/conclusion.typ"


#counter(heading).update(0)
#{
    set heading(numbering: "A.1.", supplement: "Annexe")
    show heading.where(level: 1): set heading(numbering: "A")
    include "appendices/ai.typ"
    include "appendices/full-config.typ"
    include "appendices/nix-primer.typ"
}

#set heading(numbering: none)
#bibliography("../bibliography.yaml")
