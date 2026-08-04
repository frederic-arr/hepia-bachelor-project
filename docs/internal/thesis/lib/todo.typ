#let placeholder(body) = highlight(fill: rgb("#fff091"), text(
    fill: red,
    [\<PLACEHOLDER> #body \</PLACEHOLDER>],
))

#let todo-inline(body) = highlight(fill: rgb("#fff091"), text(
    fill: red,
    [\<TODO> #body \</TODO>],
))

#let todo-ref() = highlight(fill: rgb("#fff091"), text(
    fill: red,
    [RÉFÉRENCER],
))

#let todo-chapter = highlight(fill: rgb("#fff091"), text(
    fill: red,
    [\[RÉF. CHAPITRE\]],
))

#let todo(content, ..body, label: "TODO", color: rgb("#0fb9b1")) = {
    import "../conf.typ"

    show std.figure.caption: it => {
        set align(left)
        rect(
            fill: color.lighten(80%),
            stroke: color,
            radius: 3pt,
            inset: 5pt,
            width: 100%,
        )[
            *#it.supplement:* #it.body \ #body.at(0, default: none)
        ]
    }

    if conf.ENABLE_TODO {
        std.figure(
            kind: "todo",
            supplement: label,
            caption: content,
        )[]
    }
}

#let todo-missing(content, ..body) = todo(
    content,
    ..body,
    label: "Missing",
    color: rgb("#e84393"),
)
#let todo-check(content, ..body) = todo(
    content,
    ..body,
    label: "Check",
    color: rgb("#fc5c65"),
)
#let todo-revise(content, ..body) = todo(
    content,
    ..body,
    label: "Revise",
    color: rgb("#fd9644"),
)
#let todo-citation(content, ..body) = todo(
    content,
    ..body,
    label: "Citation",
    color: rgb("#fed330"),
)
#let todo-language(content, ..body) = todo(
    content,
    ..body,
    label: "Language",
    color: rgb("#a55eea"),
)
#let todo-question(content, ..body) = todo(
    content,
    ..body,
    label: "Question",
    color: rgb("#45aaf2"),
)
#let todo-ready(content, ..body) = todo(
    content,
    ..body,
    label: "Ready",
    color: rgb("#26de81"),
)
#let todo-note(content, ..body) = todo(
    content,
    ..body,
    label: "Note",
    color: rgb("#778ca3"),
)

#let todo-outline() = {
    outline(title: none, target: std.figure.where(kind: "todo"))
}
