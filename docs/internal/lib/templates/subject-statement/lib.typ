#let subject-statement(
    title: none,
    program: none,
    header: none,
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
    description: none,
    assignment: none,
    body,
) = {
    import "/lib/_utils.typ"

    set page(margin: 2cm) // TODO: Config
    set v(weak: true)
    set text(hyphenate: false)
    set par(justify: true, first-line-indent: 0em)

    _utils.logo-header(content-right: strong(header))

    align(center, {
        block(text(size: 1.3em, weight: "bold", smallcaps(title)))
        block(text(size: 1.15em, weight: "bold", smallcaps(program)))
        v(1.5cm)
    })

    body

    v(1fr)
    _utils.meta-footer(
        author: author,
        field-of-study: field-of-study,
        supervisors: supervisors,
        client: client,
        internship: internship,
        confidentiality-agreement: confidentiality-agreement,
    )
}
