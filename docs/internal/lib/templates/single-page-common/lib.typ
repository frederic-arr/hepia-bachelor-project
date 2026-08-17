#let single-page(
    title: none,
    anchor: none,
    header: image("/lib/assets/hes-so-ge-logo.svg"),
    fonts: (
        header: "Liberation Sans",
        footer: "Liberation Sans",
    ),
    author: (
        statement: [Candidat],
        name: [ARROYO Frédéric],
    ),
    field-of-study: (
        statement: [Filière d'études],
        name: [ISC],
    ),
    supervisors: (
        statement: [Professeur responsable],
        names: [GLÜCK Florent],
    ),
    client: (
        statement: [En collaboration avec],
        name: [n/a],
    ),
    internship: (
        statement: [Travail de bachelor soumis à une convention de stage en
            entreprise],
        value: [non],
    ),
    confidentiality-agreement: (
        statement: [Travail soumis à un contrat de confidentialité],
        value: [non],
    ),
    body,
) = {
    import "/lib/_utils.typ"

    show: _utils.common-config

    if header != none {
        _utils.logo-header(content-right: strong(header))
    } else {
        v(51pt, weak: false)
    }

    align(center, {
        place(hide(heading(anchor)))
        block(text(size: 1.5em, weight: "bold", smallcaps(title)))
        v(32pt)
    })

    body

    if author != none {
        v(1fr)
        {
            set text(font: fonts.footer)
            _utils.meta-footer(
                author: author,
                field-of-study: field-of-study,
                supervisors: supervisors,
                client: client,
                internship: internship,
                confidentiality-agreement: confidentiality-agreement,
            )
        }
    }

    pagebreak(weak: true)
}
