#import "/packages.typ": *
#import "../lib.typ": *
#import packages.codly: *

#refdiagram(
    label: <ctrlloop>,
    caption: [Orchestraion de la réconciliation dans un modèle centralisé],
    note: [
        Illustre la manière donc la réconciliation individuelle de chaque
        ressource est intégrée vis-à-vis des autres ressources au sein d'un
        modèle de réconciliation centralisé.
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
