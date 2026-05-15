#let common-config(body) = {
    set page(margin: 2cm)
    set v(weak: true)
    set text(size: 12pt, lang: "fr", font: "Liberation Serif", hyphenate: false)
    show smallcaps: set text(font: "Alegreya Sans SC")
    set par(justify: true, first-line-indent: (amount: 2em, all: true))

    body
}

#let user-print-meta-title(body) = context {
    let user-print-meta-title = state("user-print-meta-title", it => {
        show heading: set text(16pt)
        block(text(size: 16pt, weight: "bold", it))
    }).get()

    user-print-meta-title(body)
}

#let meta-heading(body) = {
    user-print-meta-title(heading(body))
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
    set text(size: 0.9em)

    grid(
        columns: (1fr, 1fr),
        gutter: 12em,
        align: (left, left),
        {
            par[
                #author.statement: \ *#author.name*
            ]
            v(2em)

            par[#field-of-study.statement: #field-of-study.name]
        },
        {
            par[
                #supervisors.statement: \ *#supervisors.names*
            ]
            v(2em)

            if client != none {
                par[*#client.statement:* #client.name]
            }

            par[#internship.statement: *#internship.value*]
            par[#confidentiality-agreement.statement:
                *#confidentiality-agreement.value*]
        },
    )
}
