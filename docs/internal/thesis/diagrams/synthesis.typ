#import "/packages.typ": *
#import "../lib.typ": *
#import packages.codly: *

#refdiagram(
    label: <synthesis>,
    caption: [Réconciliation d'une ressource],
    note: [
        Le système observe l'état actuel de la ressource, calcule l'écart avec
        la spécification, et applique les actions correctives.
    ],
    source: made-by-self,

    spacing: 1cm,
    node-stroke: 2pt,
    edge-stroke: 2pt,
    mark-scale: 60%,
    {
        node(label: <synthesis-spec>, (0, 0), num: [2], title: [Specification])
        node(label: <synthesis-state>, (1, 0), num: [6], title: [State])
        node(
            label: <synthesis-rec>,
            num: [4],
            badge-fill: red,
            (0.5, 2),
            stroke: red,
            title: [Reconciliation],
        )
        node(
            label: <synthesis-phy>,
            num: [5],
            (0.5, 3),
            title: [Managed Resource],
        )
        node(
            label: <synthesis-con>,
            enclose: (<synthesis-rec>, <synthesis-phy>),
            num: [3],
            badge-x: -1.7em,
            badge-y: -2em,
            badge-fill: teal,
            inset: 5mm,
            snap: false,
            stroke: teal,
            title: place(dx: 4.5cm, dy: 2.5cm, text(fill: teal)[*Controller*]),
        )
        node(
            label: <synthesis-res>,
            enclose: (<synthesis-spec>, <synthesis-state>),
            num: [1],
            badge-x: -1.7em,
            badge-y: -2em,
            badge-fill: orange,
            inset: 5mm,
            snap: false,
            stroke: orange,
            title: place(dx: 6cm, dy: 0.9cm, text(fill: orange)[*Resource*]),
        )

        edge(
            <synthesis-spec>,
            <synthesis-rec>,
            "-|>",
            label-side: right,
            label-pos: 0.7cm,
            title: [Reconcile],
        )
        edge(
            <synthesis-rec>,
            <synthesis-state>,
            "-|>",
            label-side: right,
            label-pos: 1.5cm,
            title: [Return state],
        )
        edge(
            <synthesis-rec>,
            <synthesis-phy>,
            "<|-|>",
            title: [Gather state],
        )
    },
)
