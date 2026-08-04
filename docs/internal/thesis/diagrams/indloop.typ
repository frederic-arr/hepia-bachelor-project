#import "/packages.typ": *
#import "../lib.typ": *
#import packages.codly: *

#refdiagram(
    label: <indloop>,
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
            label: <indloop-a>,
            (0, 1),
            title: [Resource A],
            subtitle: [Reconciliation],
            stroke: red,
        )
        node(
            label: <indloop-b>,
            (1, 1),
            title: [Resource B],
            subtitle: [Reconciliation],
            stroke: red,
        )
        node(
            label: <indloop-z>,
            (3, 1),
            title: [Resource Z],
            subtitle: [Reconciliation],
            stroke: red,
        )
        edge(
            <indloop-a>,
            <indloop-a>,
            "-|>",
            title: [Infinitely recurring],
            bend: 120deg,
        )
        edge(
            <indloop-b>,
            <indloop-b>,
            "-|>",
            title: [Infinitely recurring],
            bend: 120deg,
        )
        edge(
            <indloop-z>,
            <indloop-z>,
            "-|>",
            title: [Infinitely recurring],
            bend: 120deg,
        )
    },
)
