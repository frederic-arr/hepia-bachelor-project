#let figure(
    body,
    label: none,
    caption: none,
    source: none,
    note: none,
) = {
    show std.figure.caption: it => {
        align(start)[#text(style: "italic", it)]
    }

    set par(justify: true, first-line-indent: 0em, spacing: 1em)

    show std.figure.where(kind: raw): set std.figure(supplement: "Code")
    block(
        breakable: false,
        above: 2em,
        below: 2em,
        width: 100%,
        inset: (left: 0.5cm, right: 0.5cm),
        {
            [
                #std.figure(
                    caption: caption,
                    block(width: 100%)[#body],
                ) #label
            ]

            if note != none {
                text(style: "italic")[#note]
                v(1em, weak: true)
            }

            if source != none {
                text(style: "italic")[Source: #if type(source) == label [
                        tiré de #cite(source, form: "prose")
                    ] else { source }]
                v(1em, weak: true)
            }
        },
    )
}


#let badge(n) = context {
    import "../conf.typ": is-color
    let size = measure("A").height
    box(
        width: size * 2,
        height: size,
        place(
            center + horizon,
            box(
                fill: is-color(yellow, white),
                stroke: is-color(none, 1pt),
                radius: 50%,
                width: 1.2em,
                height: 1.2em,
                align(center + horizon, text(
                    fill: black,
                    weight: "bold",
                    size: 8pt,
                    n,
                )),
            ),
        ),
    )
}


#let node(label: none, num: none, title: none, subtitle: none, ..args) = {
    import "/packages.typ": *
    import packages.fletcher: node

    node(name: label, ..args, {
        if num != none {
            context {
                let els = state("badge")
                els.update(_ => num)
            }
        }

        if subtitle == none [
            #title #label
        ] else [
            *#title*\ #text(size: 8pt)[(#subtitle)] #label
        ]

        if num != none {
            place(
                top + left,
                dx: -1.2em,
                dy: -1em,
                context {
                    let els = state("badge")
                    badge(num)
                },
            )
        }
    })
}

#let edge(label: none, num: none, title: none, subtitle: none, ..args) = {
    import "/packages.typ": *
    import packages.fletcher: edge

    let els = counter("badge")
    edge(..args, {
        if num != none {
            context {
                let els = state("badge")
                els.update(_ => num)
            }
        }

        if subtitle == none [
            #title #label
        ] else [
            *#title*\ #text(size: 8pt)[(#subtitle)] #label
        ]

        if num != none {
            place(
                top + left,
                dx: -1.2em,
                dy: -1em,
                context {
                    let els = state("badge")
                    badge(num)
                },
            )
        }
    })
}

#let bref(l) = context {
    let els = state("badge")
    badge(els.at(query(l).first().location()))
}

#let sbref(l) = context {
    let els = state("badge")
    [(#els.at(query(l).first().location()))]
}

#let refdiagram(
    label: none,
    caption: none,
    source: none,
    note: none,
    ..args,
) = context {
    import "/packages.typ": *
    import packages.fletcher: diagram

    let els = counter("badge")
    els.update(c => 1)
    figure(
        label: label,
        caption: caption,
        source: source,
        note: note,
        diagram(
            ..args,
        ),
    )
}
