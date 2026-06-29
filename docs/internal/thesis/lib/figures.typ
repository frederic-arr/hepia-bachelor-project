#let outline-figure() = {
    state("use-short-caption", false).update(_ => true)
    outline(
        title: [Table des illustrations],
        target: figure.where(kind: image),
    )
    outline(
        title: none,
        target: figure.where(kind: table),
    )
    state("use-short-caption", false).update(_ => false)
}

#let figure(
    body,
    label: none,
    caption: none,
    source: none,
    note: none,
) = {
    let flex-caption(short, long) = context {
        if state("use-short-caption", false).get() { short } else {
            long
        }
    }

    show std.figure.caption: it => {
        align(start)[#text(style: "italic", it)]
    }

    set par(justify: false, first-line-indent: 0em, spacing: 1em)

    show std.figure.where(kind: raw): set std.figure(supplement: "Code")

    block(
        breakable: false,
        above: 2em,
        below: 2em,
        width: 100%,
        inset: (left: 0.5cm, right: 0.5cm),
        [
            #std.figure(
                caption: flex-caption(caption, [
                    #caption \
                    #note \
                    #if source != none {
                        text(style: "italic")[Source: #if (
                                type(source) == label
                            ) [
                                tiré de #cite(source, form: "prose")
                            ] else { source }]
                    }
                ]),
                [#body],
            ) #label
        ],
    )
}

#let badge(n, fill: yellow, color: black) = context {
    import "../conf.typ": is-color
    let body = text(
        size: 8pt,
        fill: color,
        weight: "bold",
        font: "Liberation Mono",
        n,
    )
    let dims = measure(body)
    let w = dims.width
    let h = dims.height

    let is-single = body.child.text.len() == 1

    let pad-x = 0.6em
    let height = 1.2em
    let width = if is-single { height } else { w + 2 * pad-x }

    box(
        width: width,
        height: height,
        fill: fill,
        stroke: none,
        radius: 50%,
        align(center + horizon, body),
    )
}


#let node(
    label: none,
    num: none,
    title: none,
    subtitle: none,
    badge-x: -1.3em,
    badge-y: -1.3em,
    badge-fill: yellow,
    badge-color: black,
    ..args,
) = {
    import "/packages.typ": *
    import packages.fletcher: node

    node(name: label, ..args, {
        if num != none {
            context {
                state("badge-fill").update(_ => badge-fill)
                state("badge-color").update(_ => badge-color)
                state("badge").update(_ => num)
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
                dx: badge-x,
                dy: badge-y,
                badge(num, color: badge-color, fill: badge-fill),
            )
        }
    })
}

#let edge(
    label: none,
    num: none,
    title: none,
    subtitle: none,
    badge-x: -1.2em,
    badge-y: -1em,
    badge-fill: yellow,
    badge-color: black,
    ..args,
) = {
    import "/packages.typ": *
    import packages.fletcher: edge

    let els = counter("badge")
    edge(..args, {
        if num != none {
            context {
                state("badge-fill").update(_ => badge-fill)
                state("badge-color").update(_ => badge-color)
                state("badge").update(_ => num)
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
                dx: badge-x,
                dy: badge-y,
                badge(num, color: badge-color, fill: badge-fill),
            )
        }
    })
}

#let bref(l) = context {
    let badge-num = state("badge").at(query(l).first().location())
    let badge-fill = state("badge-fill").at(query(l).first().location())
    let badge-color = state("badge-color").at(query(l).first().location())
    let b = badge(badge-num, color: badge-color, fill: badge-fill)
    let width = measure(b).width
    box(
        width: width,
        place(dy: -0.3em, center + horizon, b),
    )
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
