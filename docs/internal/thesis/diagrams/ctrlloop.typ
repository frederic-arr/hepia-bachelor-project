#import "../lib.typ": *

#refdiagram(
    label: <ctrlloop>,
    caption: [Schéma conceptuel d'une boucle de contrôle déclarative],
    note: [
        Le contrôleur (encadré rouge) observe l'état actuel de la ressource,
        calcule l'écart avec l'état désiré, et applique les actions correctives.
    ],
    source: made-by-self,

    spacing: 1.2cm,
    node-stroke: 2pt,
    edge-stroke: 2pt,
    mark-scale: 60%,
    {
        node(label: <ctrlloop-obs>, num: [1], (0, 2), title: [Observe])
        node(label: <ctrlloop-diff>, num: [2], (1, 1), title: [Diff & Plan])
        node(label: <ctrlloop-act>, (2, 2), title: [Act])
        node(
            enclose: (<ctrlloop-obs>, <ctrlloop-diff>, <ctrlloop-act>),
            inset: 5mm,
            snap: false,
            stroke: red,
            title: align(top + left, place(dx: -5mm, dy: -10mm, text(fill: red)[
                *Dynamic Resources*
            ])),
        )
        node(
            label: <ctrlloop-cfg>,
            num: [3],
            stroke: none,
            (1, 0),
            title: [Desired State],
            subtitle: [user configuration],
        )
        node(
            label: <ctrlloop-res>,
            (1, 3),
            title: [Managed Resource],
            subtitle: [actual state],
        )

        edge(<ctrlloop-cfg>, <ctrlloop-diff>, "--|>")
        edge(
            <ctrlloop-obs>,
            <ctrlloop-diff>,
            "-|>",
            bend: 30deg,
            title: [Current state],
        )
        edge(
            label: <ctrlloop-actions>,
            num: [4],
            <ctrlloop-diff>,
            <ctrlloop-act>,
            "-|>",
            bend: 30deg,
            title: place(dx: 0.3em, box(
                fill: white,
                width: 5cm,
                outset: 2mm,
                place(dy: -0.45em)[Actions to close the gap],
            )),
        )
        edge(<ctrlloop-act>, <ctrlloop-obs>, "-|>", title: [Infinitely
            recurring])
        edge(<ctrlloop-obs>, <ctrlloop-res>, "--|>", label-side: right, title: [
            Gather information
        ])
        edge(<ctrlloop-act>, <ctrlloop-res>, "--|>", label-side: left, title: [
            Execute actions
        ])
    },
)
