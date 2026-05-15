#let single-page(
    title: none,
    header: image("/lib/assets/hes-so-ge-logo.svg"),
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
    client: none,
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

    _utils.logo-header(content-right: strong(header))

    align(center, {
        block(text(size: 1.3em, weight: "bold", smallcaps(title)))
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

    pagebreak(weak: true)
}
