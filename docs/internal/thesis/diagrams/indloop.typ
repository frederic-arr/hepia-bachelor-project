#import "/packages.typ": *
#import "../lib.typ": *
#import packages.codly: *

#refdiagram(
    label: <ctrlloop>,
    caption: [Orchestraion de la réconciliation dans un modèle décentralisé],
    note: [
        Illustre la manière donc la réconciliation individuelle de chaque
        ressource est intégrée vis-à-vis des autres ressources au sein d'un
        modèle de réconciliation décentralisé.
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
        edge(
            <ctrlloop-a>,
            <ctrlloop-a>,
            "-|>",
            title: [Infinitely recurring],
            bend: 120deg,
        )
        edge(
            <ctrlloop-b>,
            <ctrlloop-b>,
            "-|>",
            title: [Infinitely recurring],
            bend: 120deg,
        )
        edge(
            <ctrlloop-z>,
            <ctrlloop-z>,
            "-|>",
            title: [Infinitely recurring],
            bend: 120deg,
        )
    },
)
