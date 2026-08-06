#let common-config(body) = {
    set page(margin: 2cm)
    set v(weak: true)
    set text(size: 12pt, lang: "fr", font: "Liberation Serif", hyphenate: false)
    show smallcaps: set text(font: "Alegreya Sans SC")
    set par(
        justify: true,
        first-line-indent: (amount: 2em, all: false),
        leading: 0.65em,
        spacing: 1.2em,
    )
    set list(marker: [#sym.dash.em], indent: 1.5em)
    set enum(indent: 1.5em)

    body
}

#let hide-in-flow(body) = {
    hide(place(body))
}

#let logo-header(
    content-left: [#image("/lib/assets/hepia-logo.svg")],
    content-right: [#image("/lib/assets/hes-so-ge-logo.svg")],
) = {
    grid(
        columns: (1fr, 1fr),
        rows: 1.3cm,
        align: (left, right),
        content-left, content-right,
    )
    v(51pt)
}

#let meta-footer(
    author: (
        statement: none,
        name: none,
    ),
    field-of-study: (
        statement: none,
        name: none,
    ),
    supervisors: (
        statement: none,
        names: none,
    ),
    client: (
        statement: none,
        name: none,
    ),
    internship: (
        statement: none,
        value: none,
    ),
    confidentiality-agreement: (
        statement: none,
        value: none,
    ),
) = {
    set par(justify: false, first-line-indent: 0em)
    set text(size: 0.85em)

    grid(
        columns: (1fr, 1fr),
        gutter: 12em,
        align: (left, left),
        {
            if author != none {
                par[
                    #author.statement: \
                    #text(size: 1.2em, smallcaps[*#author.name*])
                ]
                v(2em)
            }

            if field-of-study != none {
                par[#field-of-study.statement: #field-of-study.name]
            }
        },
        {
            par[
                #supervisors.statement: \
                #text(size: 1.2em, smallcaps[*#supervisors.names*])
            ]
            v(2em)

            if client != none {
                par[*#client.statement:* #client.name]
            }

            if internship != none {
                par[#internship.statement: #internship.value]
            }

            if internship != none and confidentiality-agreement != none {
                v(3em)
            }

            if confidentiality-agreement != none {
                par[#confidentiality-agreement.statement:
                    #confidentiality-agreement.value]
            }
        },
    )
}
