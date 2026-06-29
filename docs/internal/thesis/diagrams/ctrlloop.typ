#import "/packages.typ": *
#import "../lib.typ": *
#import packages.codly: *

#refdiagram(
    label: <ctrlloop>,
    caption: [Système déclaratif],
    note: [
        Le système observe l'état actuel de la ressource, calcule l'écart avec
        l'état désiré, et applique les actions correctives.
    ],
    source: made-by-self,

    spacing: 1.2cm,
    node-stroke: 2pt,
    edge-stroke: 2pt,
    mark-scale: 60%,
    {
        node(
            label: <ctrlloop-a>,
            (0, 1),
            title: [Resource A],
            subtitle: [Reconciliation],
            stroke: red,
        )
        node(
            label: <ctrlloop-b>,
            (1, 1),
            title: [Resource B],
            subtitle: [Reconciliation],
            stroke: red,
        )
        node(
            label: <ctrlloop-z>,
            (3, 1),
            title: [Resource Z],
            subtitle: [Reconciliation],
            stroke: red,
        )
        edge(<ctrlloop-a>, <ctrlloop-b>, "-|>", title: [Then])
        edge(<ctrlloop-b>, <ctrlloop-z>, "--|>", title: [Then])
        edge(<ctrlloop-z>, <ctrlloop-a>, "-|>", bend: 30deg, title: [
            Infinitely recurring
        ])
    },
)
