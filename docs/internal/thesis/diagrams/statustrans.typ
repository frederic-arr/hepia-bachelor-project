#import "/packages.typ": *
#import "../lib.typ": *
#import packages.codly: *

#refdiagram(
    label: <statustrans>,
    caption: [Transition entre les différents status d'une ressource],
    note: [
        Illustre les divers status qu'une ressource peut avoir et quelles sont
        les transitions autorisée entre deux initialisation du système.
    ],
    source: made-by-self,

    spacing: 1.2cm,
    node-stroke: 2pt,
    node-shape: circle,
    edge-stroke: 2pt,
    mark-scale: 60%,
    {
        node(
            label: <statustrans-unk>,
            width: 2cm,
            (0, 1),
            title: [unknown],
            fill: aqua,
            num: [1],
            badge-fill: aqua,
            badge-y: -3em,
        )

        node(
            label: <statustrans-err>,
            width: 2cm,
            (1, 0),
            title: [error],
        )

        node(
            label: <statustrans-done>,
            width: 2cm,
            (2, 0),
            title: [done],
        )

        node(
            label: <statustrans-nrdy>,
            width: 2cm,
            (1, 2),
            title: [not ready],
        )

        node(
            label: <statustrans-rdy>,
            width: 2cm,
            (2, 2),
            title: [ready],
        )

        // node(
        //     label: <statustrans-del>,
        //     width: 2cm,
        //     (3, 1),
        //     title: [deleting],
        // )

        edge(
            label: <statustrans-to-err>,
            <statustrans-unk>,
            <statustrans-err>,
            "-|>",
            stroke: yellow,
            num: [2],
        )
        edge(
            <statustrans-unk>,
            <statustrans-rdy>,
            "-|>",
            bend: 15deg,
            stroke: yellow,
        )
        edge(<statustrans-unk>, <statustrans-nrdy>, "-|>", stroke: yellow)
        edge(<statustrans-err>, <statustrans-nrdy>, "--", stroke: red.lighten(50%))
        edge(<statustrans-err>, <statustrans-rdy>, "--", stroke: red.lighten(50%))
        edge(
            label: <statustrans-to-rdy>,
            <statustrans-rdy>,
            <statustrans-nrdy>,
            "--",
            stroke: red.lighten(50%),
            num: [3],
            badge-fill: red.lighten(50%),
            badge-y: 1em,
        )
        edge(
            <statustrans-unk>,
            <statustrans-done>,
            stroke: 2pt + teal,
            "-|>",
            bend: -15deg,
        )
        edge(<statustrans-rdy>, <statustrans-done>, stroke: 2pt + teal, "-|>")
        edge(
            <statustrans-nrdy>,
            <statustrans-done>,
            stroke: 2pt + teal,
            "-|>",
        )
        edge(
            label: <statustrans-to-done>,
            <statustrans-err>,
            <statustrans-done>,
            stroke: 2pt + teal,
            "-|>",
            num: [4],
            badge-fill: teal,
        )

        edge((-1, 1), <statustrans-unk>, "o-|>", title: [start])

        edge((-1.5, -2), (-0.5, -2), "-|>", title: [one-way transition])
        edge((-1.5, -1), (-0.5, -1), "--", title: [two-way transition])
    },
)
