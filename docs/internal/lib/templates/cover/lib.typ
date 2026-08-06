#let cover(
    title: none,

    illustration: (
        image: none,
        legend-statement: none,
        legend: none,
    ),

    submission: (
        statement: none,
        author: none,
        date: none,
    ),

    degree-statement: none,
    field-of-study: none,

    supervisors: (
        statement: none,
        supervisors: none,
    ),

    client: (
        statement: none,
        client: none,
    ),
) = {
    import "/lib/_utils.typ"

    set page(
        numbering: none,
        header: none,
        footer: none,
        margin: 2cm,
    )
    set par(justify: false, first-line-indent: 0em)
    set pagebreak(weak: true)

    {
        show: _utils.common-config
        _utils.logo-header()

        set text(font: "Liberation Sans")
        set align(center)
        set v(weak: true)

        v(51pt)

        // Title
        text(size: 1.5em, weight: "bold", title)
        // v(29pt)
        v(1fr)

        // Illustration
        block(width: 16cm)[
            #set image(fit: "contain", width: 100%)
            #if illustration != none [
                #illustration.image
            ]
        ]
        // v(51pt)
        v(1fr)

        // Presented by
        text(size: 1.3em, submission.statement)
        v(21pt)

        // John SMITH
        text(size: 1.5em, weight: "bold", smallcaps(submission.author))
        v(21pt)

        // To obtain the Bachelor of Science ...
        text(size: 1.3em, degree-statement)
        v(21pt)

        // Computer and Communication Systems ...
        text(size: 1.3em, weight: "bold", field-of-study)
        v(32pt)

        // March 2050
        text(size: 1.3em, weight: "bold", submission.date)
        v(32pt)

        // Supervisors on the left, and clients, if any, on the right
        grid(
            columns: if client == none { 1fr } else { (1fr, 1fr) },

            {
                supervisors.statement
                v(13pt)
                text(weight: "bold", size: 1.3em, smallcaps(supervisors.names))
            },

            if client != none {
                client.statement
                v(13pt)
                text(weight: "bold", client.name)
            },
        )

        pagebreak()
    }

    if illustration != none {
        v(1fr, weak: false)
        illustration.legend-statement
        linebreak()
        emph(illustration.legend)

        pagebreak()
    }
}
