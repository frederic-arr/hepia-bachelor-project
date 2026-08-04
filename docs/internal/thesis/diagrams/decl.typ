#import "/packages.typ": *
#import "../lib.typ": *
#import packages.codly: *

#refdiagram(
    label: <decl>,
    caption: [Réconciliation d'une ressource],
    note: [
        Le système observe l'état actuel de la ressource, calcule l'écart avec
        la spécification, et applique les actions correctives.
    ],
    source: made-by-self,

    spacing: 1.2cm,
    node-stroke: 2pt,
    edge-stroke: 2pt,
    mark-scale: 60%,
    {
        node(label: <decl-obs>, num: [1], (0, 2), title: [Observe])
        node(label: <decl-diff>, num: [2], (1, 1), title: [Diff & Plan])
        node(label: <decl-act>, (2, 2), title: [Act])
        node(
            enclose: (<decl-obs>, <decl-diff>, <decl-act>),
            inset: 5mm,
            snap: false,
            stroke: red,
            title: align(top + left, place(dx: -5mm, dy: -10mm, text(fill: red)[
                *Reconciliation*
            ])),
        )
        node(
            label: <decl-cfg>,
            num: [3],
            stroke: none,
            (1, 0),
            title: [Desired State],
            subtitle: [specification],
        )
        node(
            label: <decl-res>,
            (1, 3),
            title: [Managed Resource],
            subtitle: [actual state],
        )

        edge(<decl-cfg>, <decl-diff>, "--|>")
        edge(
            <decl-obs>,
            <decl-diff>,
            "-|>",
            bend: 30deg,
            title: [Current state],
        )
        edge(
            label: <decl-actions>,
            num: [4],
            <decl-diff>,
            <decl-act>,
            "-|>",
            bend: 30deg,
            title: place(dx: 0.3em, box(
                fill: white,
                width: 5cm,
                outset: 2mm,
                place(dy: -0.45em)[Actions to close the gap],
            )),
        )
        edge(<decl-obs>, <decl-res>, "--|>", label-side: right, title: [
            Gather information
        ])
        edge(<decl-act>, <decl-res>, "--|>", label-side: left, title: [
            Execute actions
        ])
    },
)
